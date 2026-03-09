use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::models::{NotificationConfig, Settings};

#[derive(Serialize, Deserialize, Default)]
struct SettingsFile {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub notification_config: NotificationConfig,
}

fn app_data_dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join("NetSentry")
}

pub fn load_settings_and_notifications() -> (Settings, NotificationConfig) {
    let path = app_data_dir().join("settings.json");
    let file: SettingsFile = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    (file.settings, file.notification_config)
}

pub fn save_settings(settings: &Settings, notification_config: &NotificationConfig) -> Result<(), String> {
    let dir = app_data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Create dir: {e}"))?;
    let file = SettingsFile {
        settings: settings.clone(),
        notification_config: notification_config.clone(),
    };
    let json = serde_json::to_string_pretty(&file).map_err(|e| format!("Serialize: {e}"))?;
    std::fs::write(dir.join("settings.json"), json).map_err(|e| format!("Write: {e}"))
}

/// Toggle the Windows startup registry entry for NetSentry via `reg.exe`.
pub fn set_autorun(enabled: bool) -> Result<(), String> {
    let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    if enabled {
        let exe = std::env::current_exe().map_err(|e| format!("Exe path: {e}"))?;
        let val = format!("\"{}\"", exe.to_string_lossy());
        let status = std::process::Command::new("reg")
            .args(["add", key, "/v", "NetSentry", "/t", "REG_SZ", "/d", &val, "/f"])
            .output()
            .map_err(|e| format!("reg add: {e}"))?
            .status;
        if !status.success() {
            return Err("reg add returned non-zero exit code".to_string());
        }
    } else {
        // Deletion is best-effort — silently ignore if the key does not exist.
        let _ = std::process::Command::new("reg")
            .args(["delete", key, "/v", "NetSentry", "/f"])
            .output();
    }
    Ok(())
}
