use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::history::HistoryEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsService {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub status: String,
    #[serde(rename = "startupType")]
    pub startup_type: String,
    pub info: ServiceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub description: Option<String>,
    pub explained: Option<String>,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupApp {
    pub name: String,
    pub command: String,
    pub location: StartupLocation,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StartupLocation {
    HkeyLocalMachine,
    HkeyCurrentUser,
    StartupFolder,
}

impl StartupLocation {
    pub fn as_str(&self) -> &'static str {
        match self {
            StartupLocation::HkeyLocalMachine => "HKLM",
            StartupLocation::HkeyCurrentUser => "HKCU",
            StartupLocation::StartupFolder => "StartupFolder",
        }
    }
}

pub struct AppState {
    pub services_cache: Mutex<ServicesCache>,
    pub services_info: Mutex<HashMap<String, ServiceInfo>>,
    pub history: Mutex<Vec<HistoryEntry>>,
    pub startup_apps_cache: Mutex<Vec<StartupApp>>,
    pub db: Mutex<rusqlite::Connection>,
}

pub struct ServicesCache {
    pub data: Vec<WindowsService>,
    pub last_updated: SystemTime,
    pub ttl: Duration,
}
