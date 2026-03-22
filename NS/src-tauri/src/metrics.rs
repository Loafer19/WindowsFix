use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

/// Lock-free performance metrics tracked during packet capture.
pub struct Metrics {
    /// Total number of packets that passed through the capture loop.
    pub packets_processed: AtomicU64,
    /// Packets dropped (blocked by rules).
    pub packets_dropped: AtomicU64,
    /// Number of capture errors encountered.
    pub capture_errors: AtomicU64,
    /// Total raw bytes seen on the network interface.
    pub bytes_seen: AtomicU64,
}

/// Serializable snapshot of current metrics.
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    #[serde(rename = "packetsProcessed")]
    pub packets_processed: u64,
    #[serde(rename = "packetsDropped")]
    pub packets_dropped: u64,
    #[serde(rename = "captureErrors")]
    pub capture_errors: u64,
    #[serde(rename = "bytesSeen")]
    pub bytes_seen: u64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            packets_processed: AtomicU64::new(0),
            packets_dropped: AtomicU64::new(0),
            capture_errors: AtomicU64::new(0),
            bytes_seen: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            packets_processed: self.packets_processed.load(Ordering::Relaxed),
            packets_dropped: self.packets_dropped.load(Ordering::Relaxed),
            capture_errors: self.capture_errors.load(Ordering::Relaxed),
            bytes_seen: self.bytes_seen.load(Ordering::Relaxed),
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
