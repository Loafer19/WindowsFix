use std::time::Duration;

/// Application configuration loaded from environment variables with sensible defaults.
///
/// AI options (key, model, timeout, max tokens) are user settings, see
/// `core::ai_settings`.
pub struct AppConfig {
    pub cache_ttl: Duration,
}

impl AppConfig {
    pub fn from_env() -> Self {
        AppConfig {
            cache_ttl: Duration::from_secs(
                std::env::var("CACHE_TTL_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300),
            ),
        }
    }
}
