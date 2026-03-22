pub mod network;
pub mod process;
pub mod settings;
pub mod windivert;

pub use network::*;
pub use process::*;
pub use settings::*;
pub use windivert::*;

use std::collections::HashMap;
use std::sync::Arc;

use crate::db;
use crate::history;
use crate::models::AppState;

/// Persist the full application state to disk.
/// Called after blocking/unblocking changes so they survive restarts.
pub fn persist_app_data(state: &Arc<AppState>) -> Result<(), String> {
    let global_hourly: Vec<(u64, u64)> =
        state.global_hourly.lock().unwrap().iter().copied().collect();
    let process_hourly: HashMap<String, Vec<(u64, u64)>> = state
        .process_hourly
        .lock()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.iter().copied().collect()))
        .collect();
    let process_totals = state.process_total_bytes.lock().unwrap().clone();
    let blocked_exes: Vec<String> =
        state.blocked_exes.lock().unwrap().iter().cloned().collect();

    db::save(&db::AppData {
        global_hourly,
        saved_at_hour: history::current_unix_hour(),
        process_totals,
        blocked_exes,
        process_hourly,
    })
}
