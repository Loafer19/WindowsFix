mod capture;
mod db;
mod history;
mod models;
mod settings;

use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use tauri::State;

use models::{AppState, HourlyPoint, NetworkStats, NotificationConfig, ProcessInfo, Settings};

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
    let pn = state.process_names.lock().unwrap().clone();
    let blocked = state.blocked_pids.lock().unwrap().clone();
    let limits = state.process_limits.lock().unwrap().clone();

    // Collect all unique PIDs from current and total
    let mut all_pids: std::collections::HashSet<u32> = pb.keys().cloned().collect();
    all_pids.extend(ptb.keys().cloned());
    all_pids.extend(blocked.iter().cloned());

    let mut list: Vec<ProcessInfo> = all_pids
        .into_iter()
        .map(|pid| {
            let (dl_bps, ul_bps) = pb.get(&pid).copied().unwrap_or((0, 0));
            let (dl_total, ul_total) = ptb.get(&pid).copied().unwrap_or((0, 0));
            ProcessInfo {
                pid,
                name: pn.get(&pid).cloned().unwrap_or_else(|| format!("PID {pid}")),
                download_bps: dl_bps,
                upload_bps: ul_bps,
                total_download_bytes: dl_total,
                total_upload_bytes: ul_total,
                blocked: blocked.contains(&pid),
                limit_bps: limits.get(&pid).copied().unwrap_or(0),
            }
        })
        .collect();

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
    state.blocked_pids.lock().unwrap().insert(pid);
    Ok(())
}

#[tauri::command]
async fn unblock_process(pid: u32, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.blocked_pids.lock().unwrap().remove(&pid);
    Ok(())
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

/// Close all TCP connections owned by `pid` by resetting them via SetTcpEntry.
/// Returns the number of connections that were successfully closed.
#[tauri::command]
async fn free_process_ports(pid: u32) -> Result<u32, String> {
    use windows::Win32::NetworkManagement::IpHelper::{
        MIB_TCPROW_LH, MIB_TCPROW_LH_0, MIB_TCP_STATE_DELETE_TCB, SetTcpEntry,
    };

    // Collect all IPv4 TCP sockets belonging to this PID via netstat2
    let sockets =
        get_sockets_info(AddressFamilyFlags::IPV4, ProtocolFlags::TCP)
            .map_err(|e| format!("get_sockets_info failed: {e}"))?;

    let mut closed = 0u32;

    for socket in &sockets {
        if !socket.associated_pids.contains(&pid) {
            continue;
        }
        let ProtocolSocketInfo::Tcp(tcp) = &socket.protocol_socket_info else {
            continue;
        };

        let (local_v4, remote_v4) = match (tcp.local_addr, tcp.remote_addr) {
            (IpAddr::V4(l), IpAddr::V4(r)) => (l, r),
            _ => continue,
        };

        // Windows MIB_TCPROW ports are stored as network-byte-order DWORDs.
        // htons(port) converts host-byte-order u16 → network-byte-order u16,
        // then we zero-extend to u32.
        let local_addr = u32::from_be_bytes(local_v4.octets());
        let remote_addr = u32::from_be_bytes(remote_v4.octets());
        let local_port = tcp.local_port.to_be() as u32;
        let remote_port = tcp.remote_port.to_be() as u32;

        unsafe {
            let mut row = MIB_TCPROW_LH {
                Anonymous: MIB_TCPROW_LH_0 {
                    dwState: MIB_TCP_STATE_DELETE_TCB.0 as u32,
                },
                dwLocalAddr: local_addr,
                dwLocalPort: local_port,
                dwRemoteAddr: remote_addr,
                dwRemotePort: remote_port,
            };
            if SetTcpEntry(&mut row) == 0 {
                closed += 1;
            }
        }
    }

    Ok(closed)
}

// ---------------------------------------------------------------------------
// 24-hour history & settings commands
// ---------------------------------------------------------------------------

#[tauri::command]
async fn get_process_history(
    pid: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<HourlyPoint>, String> {
    let hourly = state.process_hourly.lock().unwrap();
    let points = hourly
        .get(&pid)
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

    let app_state = Arc::new(AppState::new(saved_settings, saved_notif, global_hourly));

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
            free_process_ports,
            get_process_history,
            get_24h_totals,
            get_settings,
            set_settings,
            get_notification_config,
            set_notification_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
