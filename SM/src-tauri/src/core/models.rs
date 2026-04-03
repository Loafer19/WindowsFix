use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use super::history::HistoryEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    WindowsApi { message: String },
    Database { message: String },
    Config { message: String },
    Validation { message: String },
    Io { message: String },
    TaskPanic { message: String },
    Unknown { message: String },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::WindowsApi { message } => write!(f, "Windows API error: {}", message),
            AppError::Database { message } => write!(f, "Database error: {}", message),
            AppError::Config { message } => write!(f, "Configuration error: {}", message),
            AppError::Validation { message } => write!(f, "Validation error: {}", message),
            AppError::Io { message } => write!(f, "I/O error: {}", message),
            AppError::TaskPanic { message } => write!(f, "Task panic: {}", message),
            AppError::Unknown { message } => write!(f, "Unknown error: {}", message),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io {
            message: err.to_string(),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database {
            message: err.to_string(),
        }
    }
}

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
    pub path: String,
    pub arguments: Option<String>,
    pub location: StartupLocation,
    pub enabled: bool,
    pub description: Option<String>,
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
