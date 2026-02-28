use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

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

pub struct AppState {
    pub services_cache: Mutex<ServicesCache>,
    pub services_info: Mutex<HashMap<String, ServiceInfo>>,
}

pub struct ServicesCache {
    pub data: Vec<WindowsService>,
    pub last_updated: SystemTime,
    pub ttl: Duration,
}
