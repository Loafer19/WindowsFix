use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};

use windivert::prelude::WinDivertFlags;
use windivert::WinDivert;

use crate::db;
use crate::history::{current_unix_hour, HOURLY_BUCKETS};
use crate::models::AppState;

type PortPidCache = HashMap<u16, u32>;

/// Maximum size of an IPv4 or IPv6 packet in bytes (2^16 − 1).
const MAX_IP_PACKET_SIZE: usize = 65535;

fn refresh_port_pid_cache() -> PortPidCache {
    let mut cache = HashMap::new();
    if let Ok(sockets) = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP | ProtocolFlags::UDP,
    ) {
        for socket in sockets {
            let pid = match socket.associated_pids.first() {
                Some(&p) => p,
                None => continue,
            };
            let local_port = match &socket.protocol_socket_info {
                ProtocolSocketInfo::Tcp(t) => t.local_port,
                ProtocolSocketInfo::Udp(u) => u.local_port,
            };
            cache.insert(local_port, pid);
        }
    }
    cache
}

fn exe_path_for_pid(pid: u32) -> String {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return format!("PID {pid}");
        };

        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        );

        let _ = CloseHandle(handle);

        if result.is_ok() {
            String::from_utf16_lossy(&buf[..size as usize])
        } else {
            format!("PID {pid}")
        }
    }
}

pub fn capture_loop(state: Arc<AppState>) {
    let handle = loop {
        match WinDivert::network("ip", 0, WinDivertFlags::new()) {
            Ok(h) => break h,
            Err(e) => {
                eprintln!("WinDivert open failed: {e}");
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    };

    // --- Thread-Local State (No Locks needed for these) ---
    let mut pid_cache: PortPidCache = HashMap::new();
    let mut local_exe_cache: HashMap<u32, String> = HashMap::new(); // PID -> ExePath
    let mut local_blocked_pids = std::collections::HashSet::<u32>::new();

    let mut cache_refreshed = Instant::now();
    let mut window_tick = Instant::now();
    let mut current_hour = current_unix_hour();

    let mut proc_hourly_acc: HashMap<String, (u64, u64)> = HashMap::new();
    let mut global_hourly_acc: (u64, u64) = (0, 0);
    let mut recv_buf = vec![0u8; MAX_IP_PACKET_SIZE];

    loop {
        // 1. Logic Tick (Every 1 second) - Manage state and rollover
        if window_tick.elapsed() >= Duration::from_secs(1) {
            let now_hour = current_unix_hour();

            // Sync blocked PIDs and clear per-second window
            {
                let mut w = state.window.lock().unwrap();
                w.download_bytes = 0;
                w.upload_bytes = 0;
                w.start_time = std::time::Instant::now();

                let blocked = state.blocked_pids.lock().unwrap();
                local_blocked_pids = blocked.clone();

                state.process_bytes.lock().unwrap().clear();
            }

            // Hour Rollover
            if now_hour != current_hour {
                let state_clone = Arc::clone(&state);
                let proc_snap = proc_hourly_acc.clone();
                let glob_snap = global_hourly_acc;
                let old_h = current_hour;

                // Offload disk I/O to background thread
                std::thread::spawn(move || {
                    advance_hourly_background(state_clone, old_h, now_hour, proc_snap, glob_snap);
                });

                proc_hourly_acc.clear();
                global_hourly_acc = (0, 0);
                current_hour = now_hour;

                // Clear the local exe cache periodically to handle PID recycling
                local_exe_cache.clear();
            }
            window_tick = Instant::now();
        }

        // 2. Periodic Port->PID refresh
        if cache_refreshed.elapsed() > Duration::from_secs(2) {
            pid_cache = refresh_port_pid_cache();
            cache_refreshed = Instant::now();
        }

        // 3. Receive Packet
        let packet = match handle.recv(Some(&mut recv_buf)) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let is_outbound = packet.address.outbound();
        let pkt_len = packet.data.len() as u64;

        // 4. Core Filtering & Accounting
        if let Some(port) = extract_local_port(&packet.data, is_outbound) {
            if let Some(&pid) = pid_cache.get(&port) {
                if local_blocked_pids.contains(&pid) {
                    continue;
                }

                // Get Exe Path (Check local cache first, then global)
                let exe_path = if let Some(path) = local_exe_cache.get(&pid) {
                    path.clone()
                } else {
                    let path = {
                        let mut p2e = state.pid_to_exe.lock().unwrap();
                        p2e.entry(pid)
                            .or_insert_with(|| exe_path_for_pid(pid))
                            .clone()
                    };
                    local_exe_cache.insert(pid, path.clone());
                    path
                };

                // Update live stats
                {
                    let mut w = state.window.lock().unwrap();
                    if is_outbound {
                        w.upload_bytes += pkt_len;
                    } else {
                        w.download_bytes += pkt_len;
                    }
                }

                {
                    let mut pb = state.process_bytes.lock().unwrap();
                    let entry = pb.entry(pid).or_insert((0u64, 0u64));
                    if is_outbound {
                        entry.1 += pkt_len;
                    } else {
                        entry.0 += pkt_len;
                    }
                }

                if !is_outbound {
                    state.current_hour_dl.fetch_add(pkt_len, std::sync::atomic::Ordering::Relaxed);
                } else {
                    state.current_hour_ul.fetch_add(pkt_len, std::sync::atomic::Ordering::Relaxed);
                }

                {
                    let mut pa = state.current_process_acc.lock().unwrap();
                    let acc_pa = pa.entry(exe_path.clone()).or_insert((0u64, 0u64));
                    if is_outbound {
                        acc_pa.1 += pkt_len;
                    } else {
                        acc_pa.0 += pkt_len;
                    }
                }

                {
                    let mut ptb = state.process_total_bytes.lock().unwrap();
                    let tot = ptb.entry(exe_path.clone()).or_insert((0u64, 0u64));
                    if is_outbound {
                        tot.1 += pkt_len;
                    } else {
                        tot.0 += pkt_len;
                    }
                }

                // Update Hourly Accumulators
                let acc = proc_hourly_acc.entry(exe_path.clone()).or_insert((0, 0));
                if is_outbound {
                    acc.1 += pkt_len;
                } else {
                    acc.0 += pkt_len;
                }
            }
        }

        // Update Global Stats
        if is_outbound {
            global_hourly_acc.1 += pkt_len;
        } else {
            global_hourly_acc.0 += pkt_len;
        }

        handle.send(&packet).ok();
    }
}

fn advance_hourly_background(
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
    let data = {
        db::AppData {
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
        }
    };

    if let Err(e) = db::save(&data) {
        eprintln!("Disk I/O Error: {e}");
    }
}

fn extract_local_port(data: &[u8], outbound: bool) -> Option<u16> {
    if data.is_empty() {
        return None;
    }

    let version = data[0] >> 4;
    let (protocol, payload_offset) = match version {
        4 => {
            if data.len() < 20 {
                return None;
            }
            let ihl = (data[0] & 0x0F) as usize * 4;
            (data[9], ihl)
        }
        6 => {
            if data.len() < 40 {
                return None;
            }
            (data[6], 40) // Fixed header size for IPv6
        }
        _ => return None,
    };

    if data.len() < payload_offset + 4 {
        return None;
    }
    let payload = &data[payload_offset..];

    match protocol {
        6 | 17 => {
            // TCP or UDP
            let src_port = u16::from_be_bytes([payload[0], payload[1]]);
            let dst_port = u16::from_be_bytes([payload[2], payload[3]]);
            Some(if outbound { src_port } else { dst_port })
        }
        _ => None,
    }
}
