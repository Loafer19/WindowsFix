use std::collections::HashMap;

use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};

pub type PortPidCache = HashMap<u16, u32>;

/// Refresh the port → PID mapping using netstat2.
pub fn refresh_port_pid_cache() -> PortPidCache {
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

/// Resolve the executable path for a PID using Windows APIs.
pub fn exe_path_for_pid(pid: u32) -> String {
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

/// Extract the local port from a raw IP packet.
/// Returns `None` if the packet is not TCP/UDP or is malformed.
pub fn extract_local_port(data: &[u8], outbound: bool) -> Option<u16> {
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
