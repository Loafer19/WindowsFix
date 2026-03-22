use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::State;

use crate::models::{AppState, ProcessInfo};

/// Look up the cached exe path for `pid`, falling back to a descriptive string.
fn exe_path_for_pid_cached(pid: u32, state: &AppState) -> String {
    state
        .pid_to_exe
        .lock()
        .unwrap()
        .get(&pid)
        .cloned()
        .unwrap_or_else(|| format!("PID {pid}"))
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum ProcessError {
    #[error("Lock poisoned: {0}")]
    Lock(String),
    #[error("Windows API error: {0}")]
    Windows(String),
}

type ProcessResult<T> = Result<T, ProcessError>;

#[tauri::command]
pub async fn get_processes(state: State<'_, Arc<AppState>>) -> ProcessResult<Vec<ProcessInfo>> {
    let pb = state
        .process_bytes
        .lock()
        .map_err(|e| ProcessError::Lock(e.to_string()))?
        .clone();
    let ptb = state
        .process_total_bytes
        .lock()
        .map_err(|e| ProcessError::Lock(e.to_string()))?;
    let pid_to_exe = state
        .pid_to_exe
        .lock()
        .map_err(|e| ProcessError::Lock(e.to_string()))?;
    let blocked_exes = state
        .blocked_exes
        .lock()
        .map_err(|e| ProcessError::Lock(e.to_string()))?;
    let limits = state
        .process_limits
        .lock()
        .map_err(|e| ProcessError::Lock(e.to_string()))?;

    let mut list = Vec::with_capacity(ptb.len());

    // Merge persisted totals with live throughput
    for (exe_path, (dl_total, ul_total)) in ptb.iter() {
        let name = exe_path
            .split(['\\', '/'])
            .last()
            .unwrap_or(exe_path)
            .to_string();

        // Find if this exe is currently active (matching by path)
        let (pid, dl_bps, ul_bps) = pid_to_exe
            .iter()
            .find(|(_, path)| *path == exe_path)
            .map(|(&id, _)| {
                let bytes = pb.get(&id).copied().unwrap_or((0, 0));
                (id, bytes.0, bytes.1)
            })
            .unwrap_or((0, 0, 0));

        list.push(ProcessInfo {
            pid,
            name,
            exe_path: exe_path.clone(),
            download_bps: dl_bps,
            upload_bps: ul_bps,
            total_download_bytes: *dl_total,
            total_upload_bytes: *ul_total,
            blocked: blocked_exes.contains(exe_path),
            limit_bps: limits.get(&pid).copied().unwrap_or(0),
        });
    }

    list.sort_by(|a, b| {
        (b.total_download_bytes + b.total_upload_bytes)
            .cmp(&(a.total_download_bytes + a.total_upload_bytes))
    });
    Ok(list)
}

#[tauri::command]
pub async fn set_global_limit(
    bytes_per_sec: u64,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.limit_bps.store(bytes_per_sec, Ordering::Relaxed);
    let mut settings = state.settings.lock().unwrap();
    settings.global_limit_bps = bytes_per_sec;
    let notif = state.notification_config.lock().unwrap().clone();
    crate::settings::save_settings(&settings, &notif)?;
    Ok(())
}

#[tauri::command]
pub async fn set_process_limit(
    pid: u32,
    bytes_per_sec: u64,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut limits = state.process_limits.lock().unwrap();
    if bytes_per_sec == 0 {
        limits.remove(&pid);
    } else {
        limits.insert(pid, bytes_per_sec);
    }
    Ok(())
}

#[tauri::command]
pub async fn block_process(pid: u32, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let exe_path = exe_path_for_pid_cached(pid, &state);
    tracing::info!("Blocking process: {} (PID {})", exe_path, pid);

    state.blocked_exes.lock().unwrap().insert(exe_path.clone());

    // Propagate block to all currently-known PIDs with the same exe path
    let matching_pids: Vec<u32> = {
        let p2e = state.pid_to_exe.lock().unwrap();
        p2e.iter()
            .filter(|(_, v)| **v == exe_path)
            .map(|(k, _)| *k)
            .collect()
    };
    {
        let mut bpids = state.blocked_pids.lock().unwrap();
        for p in matching_pids {
            bpids.insert(p);
        }
    }

    super::persist_app_data(&state)
}

#[tauri::command]
pub async fn unblock_process(pid: u32, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let exe_path = exe_path_for_pid_cached(pid, &state);
    tracing::info!("Unblocking process: {} (PID {})", exe_path, pid);

    state.blocked_exes.lock().unwrap().remove(&exe_path);

    // Remove block from all currently-known PIDs with the same exe path
    let matching_pids: Vec<u32> = {
        let p2e = state.pid_to_exe.lock().unwrap();
        p2e.iter()
            .filter(|(_, v)| **v == exe_path)
            .map(|(k, _)| *k)
            .collect()
    };
    {
        let mut bpids = state.blocked_pids.lock().unwrap();
        for p in matching_pids {
            bpids.remove(&p);
        }
    }

    super::persist_app_data(&state)
}

#[tauri::command]
pub async fn kill_process(pid: u32) -> Result<(), String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    tracing::info!("Terminating process PID {}", pid);

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)
            .map_err(|e| format!("Failed to open process {pid}: {e}"))?;
        let result = TerminateProcess(handle, 1);
        CloseHandle(handle).ok();
        result.map_err(|e| format!("Failed to terminate process {pid}: {e}"))
    }
}
