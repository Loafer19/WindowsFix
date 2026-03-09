use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};

use windivert::prelude::WinDivertFlags;
use windivert::WinDivert;

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

fn process_name_for_pid(pid: u32) -> String {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return format!("PID {pid}"),
        };
        let mut buf = [0u16; 260];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);
        if ok.is_ok() {
            let path = String::from_utf16_lossy(&buf[..size as usize]);
            return path
                .split(['\\', '/'])
                .last()
                .unwrap_or(&path)
                .to_string();
        }
    }
    format!("PID {pid}")
}

// Returns None when bytes_per_sec is zero (meaning no limit).
// Note: governor's Quota::per_second only accepts NonZeroU32; in practice the
// maximum configurable limit is ~4.29 GB/s which covers all realistic use-cases.
fn build_limiter(bytes_per_sec: u32) -> Option<DefaultDirectRateLimiter> {
    NonZeroU32::new(bytes_per_sec).map(|n| {
        let quota = Quota::per_second(n);
        RateLimiter::direct(quota)
    })
}

pub fn capture_loop(state: Arc<AppState>) {
    let handle = match WinDivert::network("ip", 0, WinDivertFlags::new()) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("WinDivert open failed: {e}");
            eprintln!("Please ensure WinDivert is installed. Download from https://www.reqrypt.org/windivert.html");
            return;
        }
    };

    let mut pid_cache: PortPidCache = HashMap::new();
    let mut cache_refreshed = Instant::now();
    let cache_ttl = Duration::from_secs(2);

    // Global rate limiter
    let mut global_limiter: Option<DefaultDirectRateLimiter> = None;
    let mut last_global_limit: u64 = 0;

    // Per-process rate limiters (local to this thread, synced every cache refresh)
    let mut proc_limiters: HashMap<u32, DefaultDirectRateLimiter> = HashMap::new();
    let mut proc_limit_snapshot: HashMap<u32, u64> = HashMap::new();

    let mut window_tick = Instant::now();

    // Reusable receive buffer — large enough for any IP packet (max 64 KiB)
    let mut recv_buf = vec![0u8; MAX_IP_PACKET_SIZE];

    loop {
        if !state.capture_running.load(Ordering::Relaxed) {
            break;
        }

        // Reset per-second bandwidth counters
        if window_tick.elapsed() >= Duration::from_secs(1) {
            let mut w = state.window.lock().unwrap();
            w.download_bytes = 0;
            w.upload_bytes = 0;
            drop(w);
            state.process_bytes.lock().unwrap().clear();
            window_tick = Instant::now();
        }

        // Periodically refresh PID–port cache and per-process limiters
        if cache_refreshed.elapsed() > cache_ttl {
            pid_cache = refresh_port_pid_cache();

            let current_limits = state.process_limits.lock().unwrap().clone();

            // Remove stale limiters
            proc_limiters.retain(|pid, _| current_limits.contains_key(pid));
            proc_limit_snapshot.retain(|pid, _| current_limits.contains_key(pid));

            // Add or update limiters when the value changed
            for (&pid, &limit_bps) in &current_limits {
                let changed = proc_limit_snapshot.get(&pid) != Some(&limit_bps);
                if changed {
                    if let Some(lim) = build_limiter(limit_bps as u32) {
                        proc_limiters.insert(pid, lim);
                    } else {
                        proc_limiters.remove(&pid);
                    }
                    proc_limit_snapshot.insert(pid, limit_bps);
                }
            }

            cache_refreshed = Instant::now();
        }

        // Update global rate limiter when the limit changes
        let current_global = state.limit_bps.load(Ordering::Relaxed);
        if current_global != last_global_limit {
            last_global_limit = current_global;
            global_limiter = build_limiter(current_global as u32);
        }

        let packet = match handle.recv(Some(&mut recv_buf)) {
            Ok(p) => p.into_owned(),
            Err(_) => continue,
        };

        let is_outbound = packet.address.outbound();
        let pkt_len = packet.data.len() as u64;

        // Account into the current 1-second window
        {
            let mut w = state.window.lock().unwrap();
            if is_outbound {
                w.upload_bytes = w.upload_bytes.saturating_add(pkt_len);
            } else {
                w.download_bytes = w.download_bytes.saturating_add(pkt_len);
            }
        }

        // Correlate to a PID via the local TCP/UDP port
        let owner_pid = extract_local_port(&packet.data, is_outbound)
            .and_then(|port| pid_cache.get(&port).copied());

        if let Some(pid) = owner_pid {
            // Hard block: drop without re-injecting
            if state.blocked_pids.lock().unwrap().contains(&pid) {
                continue;
            }

            // Accumulate per-process bytes
            let mut pb = state.process_bytes.lock().unwrap();
            let entry = pb.entry(pid).or_insert((0, 0));
            if is_outbound {
                entry.1 = entry.1.saturating_add(pkt_len);
            } else {
                entry.0 = entry.0.saturating_add(pkt_len);
            }
            drop(pb);

            // Accumulate total per-process bytes
            let mut ptb = state.process_total_bytes.lock().unwrap();
            let total_entry = ptb.entry(pid).or_insert((0, 0));
            if is_outbound {
                total_entry.1 = total_entry.1.saturating_add(pkt_len);
            } else {
                total_entry.0 = total_entry.0.saturating_add(pkt_len);
            }
            drop(ptb);

            // Cache process name on first sight
            state
                .process_names
                .lock()
                .unwrap()
                .entry(pid)
                .or_insert_with(|| process_name_for_pid(pid));

            // Per-process token-bucket: drop if over the per-process limit
            if let Some(lim) = proc_limiters.get(&pid) {
                if let Some(n) = NonZeroU32::new(pkt_len.min(u32::MAX as u64) as u32) {
                    if !matches!(lim.check_n(n), Ok(Ok(()))) {
                        continue;
                    }
                }
            }
        }

        // Global token-bucket: drop if over the global limit
        if let Some(ref lim) = global_limiter {
            if let Some(n) = NonZeroU32::new(pkt_len.min(u32::MAX as u64) as u32) {
                if !matches!(lim.check_n(n), Ok(Ok(()))) {
                    continue;
                }
            }
        }

        if let Err(e) = handle.send(&packet) {
            eprintln!("WinDivert send failed: {e}");
        }
    }
}

fn extract_local_port(data: &[u8], outbound: bool) -> Option<u16> {
    if data.len() < 20 {
        return None;
    }
    let ihl = (data[0] & 0x0F) as usize * 4;
    if data.len() < ihl + 4 {
        return None;
    }
    let protocol = data[9];
    let payload = &data[ihl..];
    match protocol {
        6 => { // TCP
            if payload.len() >= 4 {
                let src_port = u16::from_be_bytes([payload[0], payload[1]]);
                let dst_port = u16::from_be_bytes([payload[2], payload[3]]);
                Some(if outbound { src_port } else { dst_port })
            } else {
                None
            }
        }
        17 => { // UDP
            if payload.len() >= 4 {
                let src_port = u16::from_be_bytes([payload[0], payload[1]]);
                let dst_port = u16::from_be_bytes([payload[2], payload[3]]);
                Some(if outbound { src_port } else { dst_port })
            } else {
                None
            }
        }
        _ => None,
    }
}
