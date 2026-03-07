mod capture;
mod models;

use std::net::IpAddr;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use tauri::State;

use models::{AppState, NetworkStats, ProcessInfo};

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
    let pn = state.process_names.lock().unwrap().clone();
    let blocked = state.blocked_pids.lock().unwrap().clone();
    let limits = state.process_limits.lock().unwrap().clone();

    let mut list: Vec<ProcessInfo> = pb
        .iter()
        .map(|(&pid, &(dl, ul))| ProcessInfo {
            pid,
            name: pn.get(&pid).cloned().unwrap_or_else(|| format!("PID {pid}")),
            download_bps: dl,
            upload_bps: ul,
            blocked: blocked.contains(&pid),
            limit_bps: limits.get(&pid).copied().unwrap_or(0),
        })
        .collect();

    // Include blocked-but-idle processes
    for &pid in blocked.iter() {
        if !pb.contains_key(&pid) {
            list.push(ProcessInfo {
                pid,
                name: pn.get(&pid).cloned().unwrap_or_else(|| format!("PID {pid}")),
                download_bps: 0,
                upload_bps: 0,
                blocked: true,
                limit_bps: limits.get(&pid).copied().unwrap_or(0),
            });
        }
    }

    list.sort_by(|a, b| {
        (b.download_bps + b.upload_bps).cmp(&(a.download_bps + a.upload_bps))
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
            if SetTcpEntry(&mut row).is_ok() {
                closed += 1;
            }
        }
    }

    Ok(closed)
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

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
            set_process_limit,
            block_process,
            unblock_process,
            kill_process,
            free_process_ports,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
