use std::collections::HashMap;
use std::sync::Arc;

use crate::db;
use crate::history::HOURLY_BUCKETS;
use crate::models::AppState;

/// Advance the hourly history in a background thread when an hour boundary is crossed.
/// Persists the updated data to disk after updating in-memory history.
pub fn advance_hourly_background(
    state: Arc<AppState>,
    old_hour: u64,
    new_hour: u64,
    proc_acc: HashMap<String, (u64, u64)>,
    global_acc: (u64, u64),
) {
    let hours_elapsed = (new_hour.saturating_sub(old_hour) as usize).min(HOURLY_BUCKETS);

    // 1. Update In-Memory History
    {
        let mut ph = state.process_hourly.lock().unwrap();
        let mut gh = state.global_hourly.lock().unwrap();

        for h in 0..hours_elapsed {
            let is_last = h == hours_elapsed - 1;
            let (g_dl, g_ul) = if is_last { global_acc } else { (0, 0) };

            gh.push_back((g_dl, g_ul));
            if gh.len() > HOURLY_BUCKETS {
                gh.pop_front();
            }

            for (exe, deque) in ph.iter_mut() {
                let val = if is_last {
                    proc_acc.get(exe).copied().unwrap_or((0, 0))
                } else {
                    (0, 0)
                };
                deque.push_back(val);
                if deque.len() > HOURLY_BUCKETS {
                    deque.pop_front();
                }
            }
        }
    }

    // 2. Prepare Snapshot for DB
    let data = db::AppData {
        global_hourly: state
            .global_hourly
            .lock()
            .unwrap()
            .iter()
            .copied()
            .collect(),
        saved_at_hour: new_hour,
        process_totals: state.process_total_bytes.lock().unwrap().clone(),
        blocked_exes: state.blocked_exes.lock().unwrap().iter().cloned().collect(),
        process_hourly: state
            .process_hourly
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().copied().collect()))
            .collect(),
    };

    if let Err(_e) = db::save(&data) {
        // Disk I/O error saving hourly data
    }
}
