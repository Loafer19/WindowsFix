use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use windivert::layer::NetworkLayer;
use windivert::WinDivert;

use crate::models::AppState;

type PortPidCache = HashMap<u16, u32>;

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

fn build_limiter(bytes_per_sec: u32) -> DefaultDirectRateLimiter {
    let quota = Quota::per_second(NonZeroU32::new(bytes_per_sec).unwrap());
    RateLimiter::direct(quota)
}

pub fn capture_loop(state: Arc<AppState>) {
    let handle = match WinDivert::<NetworkLayer>::new("ip") {
        Ok(h) => h,
        Err(e) => {
            eprintln!("WinDivert open failed: {e}");
            return;
        }
    };

    let mut pid_cache: PortPidCache = HashMap::new();
    let mut cache_refreshed = Instant::now();
    let cache_ttl = Duration::from_secs(2);

    let mut limiter: Option<DefaultDirectRateLimiter> = None;
    let mut last_limit: u64 = 0;

    let mut window_tick = Instant::now();

    loop {
        if !state.capture_running.load(Ordering::Relaxed) {
            break;
        }

        // Reset per-second counters every second
        if window_tick.elapsed() >= Duration::from_secs(1) {
            let mut w = state.window.lock().unwrap();
            w.download_bytes = 0;
            w.upload_bytes = 0;
            drop(w);
            state.process_bytes.lock().unwrap().clear();
            window_tick = Instant::now();
        }

        // Refresh PID–port cache periodically
        if cache_refreshed.elapsed() > cache_ttl {
            pid_cache = refresh_port_pid_cache();
            cache_refreshed = Instant::now();
        }

        // Update rate limiter when the global limit changes
        let current_limit = state.limit_bps.load(Ordering::Relaxed);
        if current_limit != last_limit {
            last_limit = current_limit;
            limiter = if current_limit > 0 {
                Some(build_limiter(current_limit as u32))
            } else {
                None
            };
        }

        let packet = match handle.recv(None) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let is_outbound = packet.address.outbound();
        let pkt_len = packet.data.len() as u64;

        // Account bandwidth in the current window
        {
            let mut w = state.window.lock().unwrap();
            if is_outbound {
                w.upload_bytes = w.upload_bytes.saturating_add(pkt_len);
            } else {
                w.download_bytes = w.download_bytes.saturating_add(pkt_len);
            }
        }

        // Correlate packet to a PID via its local TCP/UDP port
        if let Some(local_port) = extract_local_port(&packet.data, is_outbound) {
            if let Some(&pid) = pid_cache.get(&local_port) {
                // Block: drop without re-injecting
                if state.blocked_pids.lock().unwrap().contains(&pid) {
                    continue;
                }

                let mut pb = state.process_bytes.lock().unwrap();
                let entry = pb.entry(pid).or_insert((0, 0));
                if is_outbound {
                    entry.1 = entry.1.saturating_add(pkt_len);
                } else {
                    entry.0 = entry.0.saturating_add(pkt_len);
                }
                drop(pb);

                state.process_names.lock().unwrap().entry(pid).or_insert_with(|| process_name_for_pid(pid));
            }
        }

        // Apply global rate limit (token-bucket drop)
        if let Some(ref lim) = limiter {
            if let Some(n) = NonZeroU32::new(pkt_len.min(u32::MAX as u64) as u32) {
                if lim.check_n(n).is_err() {
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
    let ip4 = Ipv4Packet::new(data)?;
    match ip4.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => TcpPacket::new(ip4.payload()).map(|t| {
            if outbound { t.get_source() } else { t.get_destination() }
        }),
        IpNextHeaderProtocols::Udp => UdpPacket::new(ip4.payload()).map(|u| {
            if outbound { u.get_source() } else { u.get_destination() }
        }),
        _ => None,
    }
}
