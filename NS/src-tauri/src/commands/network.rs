use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::State;

use crate::capture;
use crate::metrics::{Metrics, MetricsSnapshot};
use crate::models::{AppState, HourlyPoint, NetworkStats};

/// Number of historical hourly points to include in the 24h view.
/// One slot is reserved for the current (incomplete) hour.
const HISTORY_HOURS: usize = 24;
const HISTORY_HOURS_PAST: usize = HISTORY_HOURS - 1;

#[tauri::command]
pub async fn start_capture(
    state: State<'_, Arc<AppState>>,
    metrics: State<'_, Arc<Metrics>>,
) -> Result<(), String> {
    let already = state.capture_running.swap(true, Ordering::Relaxed);
    if already {
        return Ok(());
    }
    let state_clone = Arc::clone(&*state);
    let metrics_clone = Arc::clone(&*metrics);
    std::thread::spawn(move || {
        capture::capture_loop(state_clone, metrics_clone);
    });
    Ok(())
}

#[tauri::command]
pub async fn stop_capture(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.capture_running.store(false, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn get_network_stats(state: State<'_, Arc<AppState>>) -> Result<NetworkStats, String> {
    let w = state.window.lock().unwrap();
    let elapsed = w.start_time.elapsed().as_secs_f64().max(0.1);
    let download_bps = ((w.download_bytes as f64 / elapsed).round()) as u64;
    let upload_bps = ((w.upload_bytes as f64 / elapsed).round()) as u64;
    Ok(NetworkStats {
        download_bps,
        upload_bps,
    })
}

#[tauri::command]
pub async fn get_24h_totals(state: State<'_, Arc<AppState>>) -> Result<HourlyPoint, String> {
    let (dl, ul) = {
        let hourly = state.global_hourly.lock().unwrap();
        hourly.iter().fold((0u64, 0u64), |(a, b), &(d, u)| (a + d, b + u))
    };
    // Include the current (incomplete) hour for real-time totals
    let current_dl = state.current_hour_dl.load(Ordering::Relaxed);
    let current_ul = state.current_hour_ul.load(Ordering::Relaxed);
    Ok(HourlyPoint {
        download_bytes: dl.saturating_add(current_dl),
        upload_bytes: ul.saturating_add(current_ul),
    })
}

#[tauri::command]
pub async fn get_process_history(
    exe_path: String,
    period: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<HourlyPoint>, String> {
    let hourly_lock = state
        .process_hourly
        .lock()
        .map_err(|_| "Failed to lock process_hourly".to_string())?;

    let raw_points: Vec<(u64, u64)> = hourly_lock
        .get(&exe_path)
        .cloned()
        .unwrap_or_default()
        .into();

    // Drop the lock early to avoid holding it during logic processing
    drop(hourly_lock);

    let current = state
        .current_process_acc
        .lock()
        .map_err(|_| "Failed to lock current_process_acc".to_string())?
        .get(&exe_path)
        .copied()
        .unwrap_or((0, 0));

    let aggregated = match period.as_str() {
        "24h" | "" => {
            let mut points: Vec<(u64, u64)> =
                raw_points.iter().rev().take(HISTORY_HOURS_PAST).copied().collect();
            points.reverse();
            points.push(current);

            if points.len() < HISTORY_HOURS {
                let mut padded = vec![(0, 0); HISTORY_HOURS - points.len()];
                padded.append(&mut points);
                padded
            } else {
                points
            }
        }
        "7d" | "30d" => {
            let days = if period == "7d" { 7 } else { 30 };
            let mut extended = raw_points;
            extended.push(current);
            aggregate_daily(&extended, days)
        }
        _ => return Err(format!("Unsupported period: {}", period)),
    };

    let points = aggregated
        .into_iter()
        .map(|(dl, ul)| HourlyPoint {
            download_bytes: dl,
            upload_bytes: ul,
        })
        .collect();

    Ok(points)
}

fn aggregate_daily(raw: &[(u64, u64)], days: usize) -> Vec<(u64, u64)> {
    let mut result: Vec<(u64, u64)> = raw
        .rchunks(24)
        .take(days)
        .map(|chunk| chunk.iter().fold((0, 0), |acc, val| (acc.0 + val.0, acc.1 + val.1)))
        .collect();

    while result.len() < days {
        result.push((0, 0));
    }

    result.reverse();
    result
}

#[tauri::command]
pub async fn get_metrics(metrics: State<'_, Arc<Metrics>>) -> Result<MetricsSnapshot, String> {
    Ok(metrics.snapshot())
}
