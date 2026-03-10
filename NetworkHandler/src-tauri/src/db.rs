use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Persisted global state that survives app restarts.
#[derive(Serialize, Deserialize, Default)]
pub struct AppData {
    /// Global rolling 24-hour download/upload history (up to 24 hourly entries).
    pub global_hourly: Vec<(u64, u64)>,
    /// Unix hour index when this snapshot was last saved.
    pub saved_at_hour: u64,
    /// Cumulative total bytes per exe path: exe_path → (dl, ul).
    #[serde(default)]
    pub process_totals: HashMap<String, (u64, u64)>,
    /// Exe paths that should remain blocked across restarts.
    #[serde(default)]
    pub blocked_exes: Vec<String>,
    /// Per-exe rolling 24-hour hourly history: exe_path → list of (dl, ul).
    #[serde(default)]
    pub process_hourly: HashMap<String, Vec<(u64, u64)>>,
}

fn data_path() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join("NetSentry").join("data.json")
}

pub fn load() -> AppData {
    let path = data_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(data: &AppData) -> Result<(), String> {
    let path = data_path();
    let dir = path
        .parent()
        .ok_or_else(|| format!("Cannot determine parent directory of '{}'", path.display()))?;
    std::fs::create_dir_all(dir).map_err(|e| format!("Dir: {e}"))?;
    let json = serde_json::to_string(data).map_err(|e| format!("JSON: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Write: {e}"))
}
