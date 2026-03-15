use std::time::{SystemTime, UNIX_EPOCH};

/// Number of hourly buckets kept in the rolling history (30 days).
pub const HOURLY_BUCKETS: usize = 720;

/// Returns the current Unix hour (elapsed seconds since epoch / 3600).
pub fn current_unix_hour() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 3600
}
