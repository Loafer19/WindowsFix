use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};

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
    #[serde(rename = "totalDownloadBytes")]
    pub total_download_bytes: u64,
    #[serde(rename = "totalUploadBytes")]
    pub total_upload_bytes: u64,
    pub blocked: bool,
    #[serde(rename = "limitBps")]
    pub limit_bps: u64,
}

/// One hour's worth of network bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyPoint {
    #[serde(rename = "downloadBytes")]
    pub download_bytes: u64,
    #[serde(rename = "uploadBytes")]
    pub upload_bytes: u64,
}

/// Persistent application settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(rename = "startWithWindows", default)]
    pub start_with_windows: bool,
    #[serde(rename = "minimizeToTray", default)]
    pub minimize_to_tray: bool,
    #[serde(rename = "globalLimitBps", default)]
    pub global_limit_bps: u64,
}

/// Notification trigger configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Fire a notification whenever a new process generates network traffic.
    #[serde(rename = "newProcessAlert", default)]
    pub new_process_alert: bool,
    /// Notify when global 24 h download exceeds this many GB (0 = disabled).
    #[serde(rename = "downloadThresholdGb", default = "default_5gb")]
    pub download_threshold_gb: f64,
    /// Notify when global 24 h upload exceeds this many GB (0 = disabled).
    #[serde(rename = "uploadThresholdGb", default = "default_5gb")]
    pub upload_threshold_gb: f64,
}

/// WinDivert installation status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinDivertStatus {
    #[serde(rename = "libraryExists")]
    pub library_exists: bool,
    #[serde(rename = "serviceExists")]
    pub service_exists: bool,
    #[serde(rename = "serviceRunning")]
    pub service_running: bool,
}

fn default_5gb() -> f64 {
    5.0
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            new_process_alert: false,
            download_threshold_gb: 5.0,
            upload_threshold_gb: 5.0,
        }
    }
}

pub struct BandwidthWindow {
    pub download_bytes: u64,
    pub upload_bytes: u64,
}

pub struct AppState {
    // ── live stats ─────────────────────────────────────────────────────────
    pub window: Arc<Mutex<BandwidthWindow>>,
    pub process_bytes: Arc<Mutex<HashMap<u32, (u64, u64)>>>,
    pub process_total_bytes: Arc<Mutex<HashMap<u32, (u64, u64)>>>,
    pub process_names: Arc<Mutex<HashMap<u32, String>>>,
    pub blocked_pids: Arc<Mutex<HashSet<u32>>>,
    pub process_limits: Arc<Mutex<HashMap<u32, u64>>>,
    pub limit_bps: Arc<AtomicU64>,
    pub capture_running: Arc<AtomicBool>,
    // ── 24-hour rolling history ─────────────────────────────────────────────
    /// Per-process hourly history: pid → deque of (dl_bytes, ul_bytes) per hour.
    pub process_hourly: Arc<Mutex<HashMap<u32, VecDeque<(u64, u64)>>>>,
    /// Global hourly history: deque of (dl_bytes, ul_bytes) per hour.
    pub global_hourly: Arc<Mutex<VecDeque<(u64, u64)>>>,
    // ── configuration ───────────────────────────────────────────────────────
    pub settings: Arc<Mutex<Settings>>,
    pub notification_config: Arc<Mutex<NotificationConfig>>,
}

impl AppState {
    pub fn new(
        settings: Settings,
        notification_config: NotificationConfig,
        global_hourly: VecDeque<(u64, u64)>,
    ) -> Self {
        Self {
            window: Arc::new(Mutex::new(BandwidthWindow {
                download_bytes: 0,
                upload_bytes: 0,
            })),
            process_bytes: Arc::new(Mutex::new(HashMap::new())),
            process_total_bytes: Arc::new(Mutex::new(HashMap::new())),
            process_names: Arc::new(Mutex::new(HashMap::new())),
            blocked_pids: Arc::new(Mutex::new(HashSet::new())),
            process_limits: Arc::new(Mutex::new(HashMap::new())),
            limit_bps: Arc::new(AtomicU64::new(settings.global_limit_bps)),
            capture_running: Arc::new(AtomicBool::new(false)),
            process_hourly: Arc::new(Mutex::new(HashMap::new())),
            global_hourly: Arc::new(Mutex::new(global_hourly)),
            settings: Arc::new(Mutex::new(settings)),
            notification_config: Arc::new(Mutex::new(notification_config)),
        }
    }
}
