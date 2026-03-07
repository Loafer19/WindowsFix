use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    #[serde(rename = "downloadBps")]
    pub download_bps: u64,
    #[serde(rename = "uploadBps")]
    pub upload_bps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    #[serde(rename = "downloadBps")]
    pub download_bps: u64,
    #[serde(rename = "uploadBps")]
    pub upload_bps: u64,
    pub blocked: bool,
}

pub struct BandwidthWindow {
    pub download_bytes: u64,
    pub upload_bytes: u64,
}

pub struct AppState {
    pub window: Arc<Mutex<BandwidthWindow>>,
    pub process_bytes: Arc<Mutex<HashMap<u32, (u64, u64)>>>,
    pub process_names: Arc<Mutex<HashMap<u32, String>>>,
    pub blocked_pids: Arc<Mutex<HashSet<u32>>>,
    pub limit_bps: Arc<AtomicU64>,
    pub capture_running: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            window: Arc::new(Mutex::new(BandwidthWindow {
                download_bytes: 0,
                upload_bytes: 0,
            })),
            process_bytes: Arc::new(Mutex::new(HashMap::new())),
            process_names: Arc::new(Mutex::new(HashMap::new())),
            blocked_pids: Arc::new(Mutex::new(HashSet::new())),
            limit_bps: Arc::new(AtomicU64::new(0)),
            capture_running: Arc::new(AtomicBool::new(false)),
        }
    }
}
