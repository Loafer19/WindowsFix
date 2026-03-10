mod capture;
mod db;
mod history;
mod models;
mod settings;

use std::collections::{HashMap, HashSet, VecDeque};

use std::sync::atomic::Ordering;
use std::sync::Arc;


use tauri::{Manager, State};

use models::{AppState, HourlyPoint, NetworkStats, NotificationConfig, ProcessInfo, Settings, WinDivertStatus};

// Embedded WinDivert driver file
static WINDRIVER_SYS: &[u8] = include_bytes!("../drivers/WinDivert64.sys");

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Persist the full application state to disk (called after blocking changes).
fn persist_app_data(state: &Arc<AppState>) -> Result<(), String> {
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

// ---------------------------------------------------------------------------
// Capture control
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Stats / process list
// ---------------------------------------------------------------------------

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
    let ptb = state.process_total_bytes.lock().unwrap().clone();
    let pid_to_exe = state.pid_to_exe.lock().unwrap().clone();
    let blocked_exes = state.blocked_exes.lock().unwrap().clone();
    let limits = state.process_limits.lock().unwrap().clone();

    // Aggregate live bytes and pick a representative PID per exe path
    let mut exe_live: HashMap<String, (u32, u64, u64)> = HashMap::new(); // exe → (pid, dl_bps, ul_bps)
    for (&pid, &(dl, ul)) in &pb {
        if let Some(exe) = pid_to_exe.get(&pid) {
            let entry = exe_live.entry(exe.clone()).or_insert((pid, 0, 0));
            entry.1 = entry.1.saturating_add(dl);
            entry.2 = entry.2.saturating_add(ul);
        }
    }
    // Ensure every known exe path is represented even when idle
    for (&pid, exe) in &pid_to_exe {
        exe_live.entry(exe.clone()).or_insert((pid, 0, 0));
    }

    // Build one ProcessInfo per exe path using persisted totals
    let mut list: Vec<ProcessInfo> = ptb
        .into_iter()
        .map(|(exe_path, (dl_total, ul_total))| {
            let name = exe_path
                .split(['\\', '/'])
                .last()
                .unwrap_or(&exe_path)
                .to_string();
            let (pid, dl_bps, ul_bps) = exe_live.get(&exe_path).copied().unwrap_or((0, 0, 0));
            ProcessInfo {
                pid,
                name,
                exe_path: exe_path.clone(),
                download_bps: dl_bps,
                upload_bps: ul_bps,
                total_download_bytes: dl_total,
                total_upload_bytes: ul_total,
                blocked: blocked_exes.contains(&exe_path),
                limit_bps: limits.get(&pid).copied().unwrap_or(0),
            }
        })
        .collect();

    // Also include currently active processes not yet in persistent totals
    for (exe_path, (pid, dl_bps, ul_bps)) in &exe_live {
        if !list.iter().any(|p| &p.exe_path == exe_path) {
            let name = exe_path
                .split(['\\', '/'])
                .last()
                .unwrap_or(exe_path)
                .to_string();
            list.push(ProcessInfo {
                pid: *pid,
                name,
                exe_path: exe_path.clone(),
                download_bps: *dl_bps,
                upload_bps: *ul_bps,
                total_download_bytes: 0,
                total_upload_bytes: 0,
                blocked: blocked_exes.contains(exe_path),
                limit_bps: limits.get(pid).copied().unwrap_or(0),
            });
        }
    }

    // Filter out processes with no historical activity
    list.retain(|p| p.total_download_bytes > 0 || p.total_upload_bytes > 0);

    list.sort_by(|a, b| {
        (b.total_download_bytes + b.total_upload_bytes).cmp(&(a.total_download_bytes + a.total_upload_bytes))
    });
    Ok(list)
}

// ---------------------------------------------------------------------------
// Traffic shaping
// ---------------------------------------------------------------------------

#[tauri::command]
async fn set_global_limit(
    bytes_per_sec: u64,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.limit_bps.store(bytes_per_sec, Ordering::Relaxed);
    let mut settings = state.settings.lock().unwrap();
    settings.global_limit_bps = bytes_per_sec;
    let notif = state.notification_config.lock().unwrap().clone();
    settings::save_settings(&settings, &notif)?;
    Ok(())
}

#[tauri::command]
async fn set_process_limit(
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
async fn block_process(pid: u32, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    // Resolve exe path for this PID
    let exe_path = {
        let p2e = state.pid_to_exe.lock().unwrap();
        p2e.get(&pid).cloned().unwrap_or_else(|| format!("PID {pid}"))
    };

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

    persist_app_data(&state)
}

#[tauri::command]
async fn unblock_process(pid: u32, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    // Resolve exe path for this PID
    let exe_path = {
        let p2e = state.pid_to_exe.lock().unwrap();
        p2e.get(&pid).cloned().unwrap_or_else(|| format!("PID {pid}"))
    };

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

    persist_app_data(&state)
}

// ---------------------------------------------------------------------------
// Process control
// ---------------------------------------------------------------------------

#[tauri::command]
async fn kill_process(pid: u32) -> Result<(), String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)
            .map_err(|e| format!("Failed to open process {pid}: {e}"))?;
        let result = TerminateProcess(handle, 1);
        CloseHandle(handle).ok();
        result.map_err(|e| format!("Failed to terminate process {pid}: {e}"))
    }
}



// ---------------------------------------------------------------------------
// 24-hour history & settings commands
// ---------------------------------------------------------------------------

#[tauri::command]
async fn get_process_history(
    exe_path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<HourlyPoint>, String> {
    let hourly = state.process_hourly.lock().unwrap();
    let points = hourly
        .get(&exe_path)
        .map(|deque| {
            deque
                .iter()
                .map(|&(dl, ul)| HourlyPoint {
                    download_bytes: dl,
                    upload_bytes: ul,
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(points)
}

#[tauri::command]
async fn get_24h_totals(state: State<'_, Arc<AppState>>) -> Result<HourlyPoint, String> {
    let hourly = state.global_hourly.lock().unwrap();
    let (dl, ul) = hourly
        .iter()
        .fold((0u64, 0u64), |(a, b), &(d, u)| (a + d, b + u));
    Ok(HourlyPoint {
        download_bytes: dl,
        upload_bytes: ul,
    })
}

#[tauri::command]
async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
    Ok(state.settings.lock().unwrap().clone())
}

#[tauri::command]
async fn set_settings(
    settings: Settings,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    settings::set_autorun(settings.start_with_windows)?;
    let notif = state.notification_config.lock().unwrap().clone();
    settings::save_settings(&settings, &notif)?;
    *state.settings.lock().unwrap() = settings;
    Ok(())
}

#[tauri::command]
async fn get_notification_config(
    state: State<'_, Arc<AppState>>,
) -> Result<NotificationConfig, String> {
    Ok(state.notification_config.lock().unwrap().clone())
}

#[tauri::command]
async fn set_notification_config(
    config: NotificationConfig,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let s = state.settings.lock().unwrap().clone();
    settings::save_settings(&s, &config)?;
    *state.notification_config.lock().unwrap() = config;
    Ok(())
}

// ---------------------------------------------------------------------------
 // WinDivert management
 // ---------------------------------------------------------------------------

#[tauri::command]
async fn check_windivert_status() -> Result<WinDivertStatus, String> {
    use std::path::Path;
    use windows::Win32::System::Services::*;

    let library_exists = Path::new(r"C:\Windows\System32\drivers\WinDivert64.sys").exists();

    let mut service_exists = false;
    let mut service_running = false;

    unsafe {
        let scm = match OpenSCManagerW(None, None, SC_MANAGER_CONNECT) {
            Ok(h) => h,
            Err(_) => return Ok(WinDivertStatus { library_exists, service_exists: false, service_running: false }),
        };

        let service_name = "WinDivert".encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
        if let Ok(service) = OpenServiceW(scm, windows::core::PCWSTR::from_raw(service_name.as_ptr()), SERVICE_QUERY_STATUS) {
            let mut status = SERVICE_STATUS::default();
            if QueryServiceStatus(service, &mut status).is_ok() {
                service_exists = true;
                service_running = status.dwCurrentState == SERVICE_RUNNING;
            }
            CloseServiceHandle(service).ok();
        }

        CloseServiceHandle(scm).ok();
    }

    Ok(WinDivertStatus { library_exists, service_exists, service_running })
}

#[tauri::command]
async fn install_windivert() -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    // Write embedded WinDivert driver to system location
    let drivers_path = Path::new(r"C:\Windows\System32\drivers\WinDivert64.sys");
    fs::write(drivers_path, WINDRIVER_SYS).map_err(|e| format!("Failed to write WinDivert64.sys to drivers: {}", e))?;

    // Create service using Windows APIs
    use windows::Win32::System::Services::*;

    unsafe {
        let scm = OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS)
            .map_err(|e| format!("Failed to open SCM: {:?}", e))?;

        let service_name = "WinDivert".encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
        let display_name = "WinDivert Packet Divert Service".encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
        let bin_path = r"\SystemRoot\System32\drivers\WinDivert64.sys".encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();

        // Delete existing service if it exists
        if let Ok(existing_service) = OpenServiceW(scm, windows::core::PCWSTR::from_raw(service_name.as_ptr()), SERVICE_ALL_ACCESS) {
            // Stop the service first
            let mut status = SERVICE_STATUS::default();
            if QueryServiceStatus(existing_service, &mut status).is_ok() && status.dwCurrentState == SERVICE_RUNNING {
                ControlService(existing_service, SERVICE_CONTROL_STOP, &mut status).ok();
            }
            // Delete the service
            DeleteService(existing_service).ok();
            CloseServiceHandle(existing_service).ok();
            // Wait a bit for deletion
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        let service = CreateServiceW(
            scm,
            windows::core::PCWSTR::from_raw(service_name.as_ptr()),
            windows::core::PCWSTR::from_raw(display_name.as_ptr()),
            SERVICE_START | SERVICE_STOP | SERVICE_QUERY_STATUS,
            SERVICE_KERNEL_DRIVER,
            SERVICE_DEMAND_START,
            SERVICE_ERROR_NORMAL,
            windows::core::PCWSTR::from_raw(bin_path.as_ptr()),
            None,
            None,
            None,
            None,
            None,
        ).map_err(|e| format!("Failed to create service: {:?}", e))?;

        // Start service
        StartServiceW(service, Some(&[])).map_err(|e| format!("Failed to start service: {:?}", e))?;

        CloseServiceHandle(service).ok();
        CloseServiceHandle(scm).ok();
    }

    Ok(())
}

#[tauri::command]
async fn start_windivert_service() -> Result<(), String> {
    use std::path::Path;
    use std::process::Command;

    // First check if the driver file exists
    if !Path::new(r"C:\Windows\System32\drivers\WinDivert64.sys").exists() {
        return Err("WinDivert64.sys not found in drivers folder. Please reinstall WinDivert.".to_string());
    }

    // Start service using sc.exe
    let output = Command::new("sc")
        .args(&["start", "WinDivert"])
        .output()
        .map_err(|e| format!("Failed to run sc start: {}", e))?;

    if !output.status.success() {
        return Err(format!("sc start failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load persisted settings and 24h global history
    let (saved_settings, saved_notif) = settings::load_settings_and_notifications();
    let saved_data = db::load();

    // Restore global hourly history, padding with zeros for any hours the app was offline
    let current_hour = history::current_unix_hour();
    let mut global_hourly: VecDeque<(u64, u64)> = saved_data.global_hourly.into_iter().collect();
    if saved_data.saved_at_hour > 0 {
        let offline_hours =
            (current_hour.saturating_sub(saved_data.saved_at_hour) as usize)
                .min(history::HOURLY_BUCKETS);
        for _ in 0..offline_hours {
            global_hourly.push_back((0, 0));
            if global_hourly.len() > history::HOURLY_BUCKETS {
                global_hourly.pop_front();
            }
        }
    }

    // Restore per-exe hourly history, padding missed hours with zeros
    let mut process_hourly: HashMap<String, VecDeque<(u64, u64)>> = saved_data
        .process_hourly
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect();
    if saved_data.saved_at_hour > 0 {
        let offline_hours =
            (current_hour.saturating_sub(saved_data.saved_at_hour) as usize)
                .min(history::HOURLY_BUCKETS);
        for deque in process_hourly.values_mut() {
            for _ in 0..offline_hours {
                deque.push_back((0, 0));
                if deque.len() > history::HOURLY_BUCKETS {
                    deque.pop_front();
                }
            }
        }
    }

    // Restore blocked exe paths
    let blocked_exes: HashSet<String> = saved_data.blocked_exes.into_iter().collect();

    let app_state = Arc::new(AppState::new(
        saved_settings,
        saved_notif,
        global_hourly,
        saved_data.process_totals,
        blocked_exes,
        process_hourly,
    ));

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};

            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("NetSentry")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Hide main window on startup if "Start minimized" is configured
            let state = app.state::<Arc<AppState>>();
            if state.settings.lock().unwrap().start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_capture,
            stop_capture,
            get_network_stats,
            get_processes,
            set_global_limit,
            set_process_limit,
            block_process,
            unblock_process,
            kill_process,
            get_process_history,
            get_24h_totals,
            get_settings,
            set_settings,
            get_notification_config,
            set_notification_config,
            check_windivert_status,
            install_windivert,
            start_windivert_service,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
