use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current Unix timestamp in seconds.
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// A unified history entry covering both service and startup-app changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HistoryEntry {
    Service(ServiceHistoryEntry),
    StartupApp(StartupAppHistoryEntry),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceHistoryEntry {
    pub service_name: String,
    pub action: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupAppHistoryEntry {
    pub app_name: String,
    pub action: String,
    pub location: String,
    pub command: Option<String>,
    pub timestamp: u64,
}

impl HistoryEntry {
    pub fn timestamp(&self) -> u64 {
        match self {
            HistoryEntry::Service(e) => e.timestamp,
            HistoryEntry::StartupApp(e) => e.timestamp,
        }
    }

    pub fn service(
        name: &str,
        action: &str,
        old_value: Option<&str>,
        new_value: Option<&str>,
    ) -> Self {
        HistoryEntry::Service(ServiceHistoryEntry {
            service_name: name.to_string(),
            action: action.to_string(),
            old_value: old_value.map(|s| s.to_string()),
            new_value: new_value.map(|s| s.to_string()),
            timestamp: current_timestamp(),
        })
    }

    pub fn startup_app(name: &str, action: &str, location: &str, command: Option<&str>) -> Self {
        HistoryEntry::StartupApp(StartupAppHistoryEntry {
            app_name: name.to_string(),
            action: action.to_string(),
            location: location.to_string(),
            command: command.map(|s| s.to_string()),
            timestamp: current_timestamp(),
        })
    }
}
