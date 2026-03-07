mod capture;
mod models;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::State;

use models::{AppState, NetworkStats, ProcessInfo};

#[tauri::command]
async fn start_capture(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let already = state.capture_running.swap(true, Ordering::Relaxed);
    if already {
        return Ok(());
    }

    let state_clone = Arc::clone(&*state);
    std::thread::spawn(move || {
        capture::capture_loop(state_clone);
    });

    Ok(())
}

#[tauri::command]
async fn stop_capture(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.capture_running.store(false, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
async fn get_network_stats(state: State<'_, Arc<AppState>>) -> Result<NetworkStats, String> {
    let w = state.window.lock().unwrap();
    Ok(NetworkStats {
        download_bps: w.download_bytes,
        upload_bps: w.upload_bytes,
    })
}

#[tauri::command]
async fn get_processes(state: State<'_, Arc<AppState>>) -> Result<Vec<ProcessInfo>, String> {
    let pb = state.process_bytes.lock().unwrap().clone();
    let pn = state.process_names.lock().unwrap().clone();
    let blocked = state.blocked_pids.lock().unwrap().clone();

    let mut list: Vec<ProcessInfo> = pb
        .iter()
        .map(|(&pid, &(dl, ul))| ProcessInfo {
            pid,
            name: pn.get(&pid).cloned().unwrap_or_else(|| format!("PID {pid}")),
            download_bps: dl,
            upload_bps: ul,
            blocked: blocked.contains(&pid),
        })
        .collect();

    // Also include blocked processes even if currently idle
    for &pid in blocked.iter() {
        if !pb.contains_key(&pid) {
            list.push(ProcessInfo {
                pid,
                name: pn.get(&pid).cloned().unwrap_or_else(|| format!("PID {pid}")),
                download_bps: 0,
                upload_bps: 0,
                blocked: true,
            });
        }
    }

    list.sort_by(|a, b| {
        (b.download_bps + b.upload_bps).cmp(&(a.download_bps + a.upload_bps))
    });

    Ok(list)
}

#[tauri::command]
async fn set_global_limit(
    bytes_per_sec: u64,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.limit_bps.store(bytes_per_sec, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
async fn block_process(pid: u32, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.blocked_pids.lock().unwrap().insert(pid);
    Ok(())
}

#[tauri::command]
async fn unblock_process(pid: u32, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.blocked_pids.lock().unwrap().remove(&pid);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(AppState::new());

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_capture,
            stop_capture,
            get_network_stats,
            get_processes,
            set_global_limit,
            block_process,
            unblock_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
