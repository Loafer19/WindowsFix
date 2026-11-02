//! AI (xAI / Grok) configuration.
//!
//! * The API key is a secret: it is stored only in the OS credential store
//!   (Windows Credential Manager via the `keyring` crate) and never written to
//!   SQLite, JSON or logs. For development a `GROK_API_KEY` / `XAI_API_KEY`
//!   environment variable is used as a fallback.
//! * Non-secret options (model, timeout, max tokens) live in the SQLite
//!   `settings` table.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::database;
use super::models::AppError;

/// Current fast, inexpensive xAI text model (see https://docs.x.ai/developers/models)
pub const DEFAULT_MODEL: &str = "grok-4.3";
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const DEFAULT_MAX_TOKENS: u32 = 1000;

pub const MIN_TIMEOUT_SECS: u64 = 5;
pub const MAX_TIMEOUT_SECS: u64 = 120;
pub const MIN_MAX_TOKENS: u32 = 100;
pub const MAX_MAX_TOKENS: u32 = 8000;

/// Environment variables checked (in order) when no key is stored. Dev only.
pub const ENV_KEY_VARS: [&str; 2] = ["GROK_API_KEY", "XAI_API_KEY"];

const KEYRING_SERVICE: &str = "WindowsFix.ServicesManager";
const KEYRING_USER: &str = "xai-api-key";
const SETTINGS_KEY: &str = "ai";

// ─── Non-secret settings ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub model: String,
    pub timeout_secs: u64,
    pub max_tokens: u32,
}

impl Default for AiSettings {
    fn default() -> Self {
        AiSettings {
            model: DEFAULT_MODEL.to_string(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            max_tokens: DEFAULT_MAX_TOKENS,
        }
    }
}

impl AiSettings {
    /// Defaults, optionally overridden by the legacy dev env vars.
    pub fn from_env_or_default() -> Self {
        let mut s = AiSettings::default();
        if let Some(model) = std::env::var("GROK_MODEL").ok().filter(|m| is_valid_model_id(m.trim())) {
            s.model = model.trim().to_string();
        }
        if let Some(t) = std::env::var("GROK_API_TIMEOUT").ok().and_then(|v| v.trim().parse().ok()) {
            s.timeout_secs = t;
        }
        if let Some(t) = std::env::var("GROK_MAX_TOKENS").ok().and_then(|v| v.trim().parse().ok()) {
            s.max_tokens = t;
        }
        s.clamped()
    }

    fn clamped(mut self) -> Self {
        self.timeout_secs = self.timeout_secs.clamp(MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS);
        self.max_tokens = self.max_tokens.clamp(MIN_MAX_TOKENS, MAX_MAX_TOKENS);
        self
    }

    /// Validate user input, returning a normalized copy.
    pub fn validated(&self) -> Result<Self, AppError> {
        let model = self.model.trim();
        if !is_valid_model_id(model) {
            return Err(AppError::Validation {
                message: "Model name may only contain letters, digits, dots, dashes and underscores".into(),
            });
        }
        if !(MIN_TIMEOUT_SECS..=MAX_TIMEOUT_SECS).contains(&self.timeout_secs) {
            return Err(AppError::Validation {
                message: format!("Timeout must be between {} and {} seconds", MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS),
            });
        }
        if !(MIN_MAX_TOKENS..=MAX_MAX_TOKENS).contains(&self.max_tokens) {
            return Err(AppError::Validation {
                message: format!("Max tokens must be between {} and {}", MIN_MAX_TOKENS, MAX_MAX_TOKENS),
            });
        }
        Ok(AiSettings { model: model.to_string(), timeout_secs: self.timeout_secs, max_tokens: self.max_tokens })
    }
}

/// Model ids are used in a URL path (`/v1/models/{id}`), so keep them strict.
pub fn is_valid_model_id(model: &str) -> bool {
    !model.is_empty()
        && model.len() <= 100
        && model.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

pub fn load_settings(conn: &Connection) -> AiSettings {
    match database::get_setting(conn, SETTINGS_KEY) {
        Ok(Some(json)) => match serde_json::from_str::<AiSettings>(&json) {
            Ok(s) => s.validated().unwrap_or_else(|_| AiSettings::from_env_or_default()),
            Err(e) => {
                eprintln!("Warning: invalid AI settings in database, using defaults: {}", e);
                AiSettings::from_env_or_default()
            }
        },
        Ok(None) => AiSettings::from_env_or_default(),
        Err(e) => {
            eprintln!("Warning: could not read AI settings: {}", e);
            AiSettings::from_env_or_default()
        }
    }
}

pub fn save_settings(conn: &Connection, settings: &AiSettings) -> Result<AiSettings, AppError> {
    let s = settings.validated()?;
    let json = serde_json::to_string(&s).map_err(|e| AppError::Io { message: format!("Failed to serialize AI settings: {}", e) })?;
    database::set_setting(conn, SETTINGS_KEY, &json)?;
    Ok(s)
}

// ─── API key ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KeySource {
    CredentialManager,
    Environment,
}

/// Abstraction over the secure store so the resolution logic is testable.
pub trait SecretStore {
    fn get(&self) -> Result<Option<String>, AppError>;
    fn set(&self, secret: &str) -> Result<(), AppError>;
    fn delete(&self) -> Result<(), AppError>;
}

/// Windows Credential Manager (generic credential "WindowsFix.ServicesManager")
pub struct KeyringStore;

impl KeyringStore {
    fn entry() -> Result<keyring::Entry, AppError> {
        keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| AppError::Config {
            message: format!("Credential Manager is unavailable: {}", e),
        })
    }
}

impl SecretStore for KeyringStore {
    fn get(&self) -> Result<Option<String>, AppError> {
        match Self::entry()?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Config { message: format!("Could not read API key from Credential Manager: {}", e) }),
        }
    }

    fn set(&self, secret: &str) -> Result<(), AppError> {
        Self::entry()?
            .set_password(secret)
            .map_err(|e| AppError::Config { message: format!("Could not save API key to Credential Manager: {}", e) })
    }

    fn delete(&self) -> Result<(), AppError> {
        match Self::entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Config { message: format!("Could not remove API key from Credential Manager: {}", e) }),
        }
    }
}

fn non_empty(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Stored key wins; env var is the dev fallback.
pub fn resolve_api_key(stored: Option<String>, env: Option<String>) -> Option<(String, KeySource)> {
    non_empty(stored)
        .map(|k| (k, KeySource::CredentialManager))
        .or_else(|| non_empty(env).map(|k| (k, KeySource::Environment)))
}

pub fn env_api_key() -> Option<String> {
    ENV_KEY_VARS.iter().find_map(|name| non_empty(std::env::var(name).ok()))
}

/// Read the effective key. A broken credential store is logged and treated as
/// "no stored key" so the env fallback still works.
pub fn current_api_key(store: &dyn SecretStore) -> Option<(String, KeySource)> {
    let stored = store.get().unwrap_or_else(|e| {
        eprintln!("Warning: {}", e);
        None
    });
    resolve_api_key(stored, env_api_key())
}

/// Basic sanity checks for a pasted key. Does not contact the API.
pub fn validate_api_key(raw: &str) -> Result<String, AppError> {
    let key = raw.trim();
    if key.is_empty() {
        return Err(AppError::Validation { message: "API key is empty".into() });
    }
    if key.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(AppError::Validation { message: "API key must not contain spaces or line breaks".into() });
    }
    if key.len() < 20 || key.len() > 512 {
        return Err(AppError::Validation { message: "API key length looks wrong, copy it again from console.x.ai".into() });
    }
    Ok(key.to_string())
}

/// Display-safe version of a key, e.g. `xai-…9f3a`
pub fn mask_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 8 {
        return "••••".to_string();
    }
    let prefix: String = if key.starts_with("xai-") { "xai-".into() } else { String::new() };
    let suffix: String = chars[chars.len() - 4..].iter().collect();
    format!("{}…{}", prefix, suffix)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub configured: bool,
    pub source: Option<KeySource>,
    pub masked_key: Option<String>,
    /// True when an env var key exists (so clearing the stored key keeps AI working)
    pub env_key_present: bool,
    pub settings: AiSettings,
}

pub fn build_status(store: &dyn SecretStore, settings: AiSettings) -> AiStatus {
    let resolved = current_api_key(store);
    AiStatus {
        configured: resolved.is_some(),
        source: resolved.as_ref().map(|(_, s)| *s),
        masked_key: resolved.as_ref().map(|(k, _)| mask_key(k)),
        env_key_present: env_api_key().is_some(),
        settings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct MemStore(RefCell<Option<String>>);
    impl SecretStore for MemStore {
        fn get(&self) -> Result<Option<String>, AppError> { Ok(self.0.borrow().clone()) }
        fn set(&self, s: &str) -> Result<(), AppError> { *self.0.borrow_mut() = Some(s.into()); Ok(()) }
        fn delete(&self) -> Result<(), AppError> { *self.0.borrow_mut() = None; Ok(()) }
    }

    struct BrokenStore;
    impl SecretStore for BrokenStore {
        fn get(&self) -> Result<Option<String>, AppError> { Err(AppError::Config { message: "boom".into() }) }
        fn set(&self, _: &str) -> Result<(), AppError> { Err(AppError::Config { message: "boom".into() }) }
        fn delete(&self) -> Result<(), AppError> { Err(AppError::Config { message: "boom".into() }) }
    }

    #[test]
    fn stored_key_wins_over_env() {
        let r = resolve_api_key(Some("stored".into()), Some("env".into()));
        assert_eq!(r, Some(("stored".into(), KeySource::CredentialManager)));
    }

    #[test]
    fn env_is_fallback_and_blank_values_ignored() {
        assert_eq!(resolve_api_key(None, Some(" env ".into())), Some(("env".into(), KeySource::Environment)));
        assert_eq!(resolve_api_key(Some("   ".into()), Some("env".into())), Some(("env".into(), KeySource::Environment)));
        assert_eq!(resolve_api_key(None, Some("".into())), None);
        assert_eq!(resolve_api_key(None, None), None);
    }

    #[test]
    fn mem_store_roundtrip_and_broken_store_does_not_panic() {
        let s = MemStore::default();
        s.set("xai-abcdefghijklmnopqrstuvwxyz").unwrap();
        assert_eq!(s.get().unwrap().as_deref(), Some("xai-abcdefghijklmnopqrstuvwxyz"));
        s.delete().unwrap();
        assert_eq!(s.get().unwrap(), None);
        // Broken store must degrade to "no stored key", not panic
        let r = current_api_key(&BrokenStore);
        assert!(r.is_none() || r.map(|(_, src)| src) == Some(KeySource::Environment));
    }

    #[test]
    fn validates_keys() {
        assert!(validate_api_key("").is_err());
        assert!(validate_api_key("short").is_err());
        assert!(validate_api_key("xai-abc def ghi jkl mno pqr").is_err());
        assert_eq!(validate_api_key("  xai-abcdefghijklmnopqrstuvwxyz\n").unwrap(), "xai-abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn masks_keys() {
        assert_eq!(mask_key("xai-abcdefghijklmnopqrstuvwxyz1234"), "xai-…1234");
        assert_eq!(mask_key("abcdefghijkl"), "…ijkl");
        assert_eq!(mask_key("short"), "••••");
        assert!(!mask_key("xai-SECRETSECRETSECRET9999").contains("SECRET"));
    }

    #[test]
    fn validates_settings() {
        let ok = AiSettings { model: " grok-4.3 ".into(), timeout_secs: 30, max_tokens: 1000 };
        assert_eq!(ok.validated().unwrap().model, "grok-4.3");
        assert!(AiSettings { model: "../etc".into(), ..AiSettings::default() }.validated().is_err());
        assert!(AiSettings { model: "".into(), ..AiSettings::default() }.validated().is_err());
        assert!(AiSettings { timeout_secs: 1, ..AiSettings::default() }.validated().is_err());
        assert!(AiSettings { max_tokens: 99_999, ..AiSettings::default() }.validated().is_err());
    }

    #[test]
    fn settings_persist_in_db() {
        let conn = database::open_memory_db().unwrap();
        let custom = AiSettings { model: "grok-4.7".into(), timeout_secs: 60, max_tokens: 2000 };
        save_settings(&conn, &custom).unwrap();
        assert_eq!(load_settings(&conn), custom);
        assert!(save_settings(&conn, &AiSettings { timeout_secs: 0, ..custom }).is_err());
    }

    #[test]
    fn default_model_is_not_outdated() {
        assert_ne!(DEFAULT_MODEL, "grok-3-mini");
        assert!(is_valid_model_id(DEFAULT_MODEL));
    }
}
