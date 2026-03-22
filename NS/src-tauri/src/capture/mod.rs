mod batch_updater;
pub mod packet_processor;

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use windivert::prelude::WinDivertFlags;
use windivert::WinDivert;

use crate::history::current_unix_hour;
use crate::metrics::Metrics;
use crate::models::AppState;
use packet_processor::{exe_path_for_pid, extract_local_port, refresh_port_pid_cache, PortPidCache};

/// Maximum size of an IPv4 or IPv6 packet in bytes (2^16 − 1).
const MAX_IP_PACKET_SIZE: usize = 65535;

/// Main packet capture loop. Runs in its own thread, retrying on WinDivert open failure.
pub fn capture_loop(state: Arc<AppState>, metrics: Arc<Metrics>) {
    let handle = loop {
        match WinDivert::network("ip", 0, WinDivertFlags::new()) {
            Ok(h) => break h,
            Err(e) => {
                tracing::warn!("WinDivert open failed: {e}; retrying in 5s");
                metrics.capture_errors.fetch_add(1, Ordering::Relaxed);
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    };

    tracing::info!("Capture loop started");

    // --- Thread-Local State (No Locks needed for these) ---
    let mut pid_cache: PortPidCache = HashMap::new();
    let mut local_exe_cache: HashMap<u32, String> = HashMap::new();
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
                    batch_updater::advance_hourly_background(
                        state_clone, old_h, now_hour, proc_snap, glob_snap,
                    );
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
            Err(e) => {
                metrics.capture_errors.fetch_add(1, Ordering::Relaxed);
                tracing::debug!("Packet recv error: {e}");
                continue;
            }
        };

        let is_outbound = packet.address.outbound();
        let pkt_len = packet.data.len() as u64;

        metrics.bytes_seen.fetch_add(pkt_len, Ordering::Relaxed);

        // 4. Core Filtering & Accounting
        if let Some(port) = extract_local_port(&packet.data, is_outbound) {
            if let Some(&pid) = pid_cache.get(&port) {
                if local_blocked_pids.contains(&pid) {
                    metrics.packets_dropped.fetch_add(1, Ordering::Relaxed);
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
                    state.current_hour_dl.fetch_add(pkt_len, Ordering::Relaxed);
                } else {
                    state.current_hour_ul.fetch_add(pkt_len, Ordering::Relaxed);
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
                let acc = proc_hourly_acc.entry(exe_path).or_insert((0, 0));
                if is_outbound {
                    acc.1 += pkt_len;
                } else {
                    acc.0 += pkt_len;
                }

                metrics.packets_processed.fetch_add(1, Ordering::Relaxed);
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
