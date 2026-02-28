use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::System::Services::*;

use crate::models::{ServiceInfo, WindowsService};

pub fn services_json_path() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().and_then(|p| p.parent()).map(|p| p.join("services.json")))
}

fn service_status_str(state: SERVICE_STATUS_CURRENT_STATE) -> &'static str {
    match state {
        SERVICE_RUNNING => "Running",
        SERVICE_STOPPED => "Stopped",
        SERVICE_START_PENDING => "Start Pending",
        SERVICE_STOP_PENDING => "Stop Pending",
        SERVICE_PAUSE_PENDING => "Pause Pending",
        SERVICE_PAUSED => "Paused",
        _ => "Unknown",
    }
}

fn startup_type_str(start_type: SERVICE_START_TYPE) -> &'static str {
    match start_type {
        SERVICE_AUTO_START => "Automatic",
        SERVICE_DEMAND_START => "Manual",
        SERVICE_DISABLED => "Disabled",
        SERVICE_BOOT_START => "Boot",
        SERVICE_SYSTEM_START => "System",
        _ => "Unknown",
    }
}

pub fn get_default_service_info(service_name: &str, defaults: &HashMap<String, ServiceInfo>) -> ServiceInfo {
    if let Some(info) = defaults.get(service_name) {
        return info.clone();
    }

    let service_lower = service_name.to_lowercase();
    for (key, info) in defaults {
        if service_lower.contains(&key.to_lowercase()) || key.to_lowercase().contains(&service_lower) {
            return info.clone();
        }
    }

    ServiceInfo {
        description: Some(format!("Windows service: {}", service_name)),
        explained: Some(format!("Windows system service '{}'. Performs specific OS functions. Use reload for AI-generated detailed explanation.", service_name)),
        recommendation: Some("• Research service function before changes\n• Many services are essential for stability\n• Consider functionality impact before disabling\n• Use reload button for detailed AI analysis".to_string()),
    }
}

pub async fn get_windows_services() -> Result<Vec<WindowsService>, String> {
    unsafe {
        let scm = OpenSCManagerW(None, None, SC_MANAGER_ENUMERATE_SERVICE)
            .map_err(|e| format!("Failed to open SCM: {:?}", e))?;

        let mut bytes_needed: u32 = 0;
        let mut services_returned: u32 = 0;

        let result = EnumServicesStatusExW(
            scm,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32,
            SERVICE_STATE_ALL,
            None,
            &mut bytes_needed,
            &mut services_returned,
            None,
            None,
        );

        if let Err(e) = result {
            // ERROR_MORE_DATA (0x800700EA) is expected when buffer is too small
            if e.code().0 != 0x800700EAu32 as i32 {
                CloseServiceHandle(scm).ok();
                return Err(format!("Failed to get buffer size: {:?}", e));
            }
        }

        if bytes_needed == 0 {
            CloseServiceHandle(scm).ok();
            return Ok(vec![]);
        }

        let mut buffer = vec![0u8; bytes_needed as usize];

        loop {
            let result = EnumServicesStatusExW(
                scm,
                SC_ENUM_PROCESS_INFO,
                SERVICE_WIN32,
                SERVICE_STATE_ALL,
                Some(&mut buffer),
                &mut bytes_needed,
                &mut services_returned,
                None,
                None,
            );

            if let Err(e) = result {
                if e.code().0 == 0x800700EAu32 as i32 {
                    bytes_needed *= 2;
                    buffer.resize(bytes_needed as usize, 0);
                    continue;
                } else {
                    CloseServiceHandle(scm).ok();
                    return Err(format!("Failed to enumerate services: {:?}", e));
                }
            } else {
                break;
            }
        }

        let service_infos = std::slice::from_raw_parts(
            buffer.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW,
            services_returned as usize,
        );

        let mut services = Vec::new();

        for service_info in service_infos {
            let name = service_info.lpServiceName.to_string().map_err(|_| "Invalid service name")?;
            let mut display_name = service_info.lpDisplayName.to_string().map_err(|_| "Invalid display name")?;
            if display_name.is_empty() {
                display_name = name.clone();
            }

            let status = service_status_str(service_info.ServiceStatusProcess.dwCurrentState).to_string();

            let startup_type = if let Ok(service) = OpenServiceW(
                scm,
                PCWSTR::from_raw(service_info.lpServiceName.as_ptr() as *const _),
                SERVICE_QUERY_CONFIG,
            ) {
                let mut config_size: u32 = 0;
                let _ = QueryServiceConfigW(service, None, 0, &mut config_size);

                let st = if config_size > 0 {
                    let mut config_buffer = vec![0u8; config_size as usize];
                    if QueryServiceConfigW(
                        service,
                        Some(config_buffer.as_mut_ptr() as *mut _),
                        config_size,
                        &mut config_size,
                    )
                    .is_ok()
                    {
                        let config = &*(config_buffer.as_ptr() as *const QUERY_SERVICE_CONFIGW);
                        startup_type_str(config.dwStartType)
                    } else {
                        "Unknown"
                    }
                } else {
                    "Unknown"
                };
                CloseServiceHandle(service).ok();
                st.to_string()
            } else {
                "Unknown".to_string()
            };

            services.push(WindowsService {
                name,
                display_name,
                status,
                startup_type,
                info: ServiceInfo {
                    description: None,
                    explained: None,
                    recommendation: None,
                },
            });
        }

        CloseServiceHandle(scm).ok();
        Ok(services)
    }
}

pub fn disable_windows_service(service_name: &str) -> Result<WindowsService, String> {
    unsafe {
        let scm = OpenSCManagerW(None, None, SC_MANAGER_CONNECT)
            .map_err(|e| format!("Failed to open SCM: {:?}", e))?;

        let service_name_wide = service_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        let service = OpenServiceW(
            scm,
            PCWSTR::from_raw(service_name_wide.as_ptr()),
            SERVICE_CHANGE_CONFIG | SERVICE_STOP | SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG,
        )
        .map_err(|e| format!("Failed to open service: {:?}", e))?;

        let mut status = SERVICE_STATUS::default();
        if QueryServiceStatus(service, &mut status).is_ok() && status.dwCurrentState == SERVICE_RUNNING {
            ControlService(service, SERVICE_CONTROL_STOP, &mut status).ok();
        }

        ChangeServiceConfigW(
            service,
            SERVICE_WIN32_OWN_PROCESS,
            SERVICE_DISABLED,
            SERVICE_ERROR_NORMAL,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .map_err(|e| format!("Failed to disable service: {:?}", e))?;

        let mut status = SERVICE_STATUS::default();
        let status_str = if QueryServiceStatus(service, &mut status).is_ok() {
            service_status_str(status.dwCurrentState).to_string()
        } else {
            "Unknown".to_string()
        };

        let mut config_size: u32 = 0;
        let _ = QueryServiceConfigW(service, None, 0, &mut config_size);
        let mut config_buffer = vec![0u8; config_size as usize];
        let startup_type = if QueryServiceConfigW(
            service,
            Some(config_buffer.as_mut_ptr() as *mut _),
            config_size,
            &mut config_size,
        )
        .is_ok()
        {
            let config = &*(config_buffer.as_ptr() as *const QUERY_SERVICE_CONFIGW);
            startup_type_str(config.dwStartType).to_string()
        } else {
            "Unknown".to_string()
        };

        CloseServiceHandle(service).ok();
        CloseServiceHandle(scm).ok();

        Ok(WindowsService {
            name: service_name.to_string(),
            display_name: service_name.to_string(),
            status: status_str,
            startup_type,
            info: ServiceInfo {
                description: None,
                explained: None,
                recommendation: None,
            },
        })
    }
}

pub fn save_services_info(services_info: &HashMap<String, ServiceInfo>) {
    if let Some(file_path) = services_json_path() {
        match serde_json::to_string_pretty(services_info) {
            Ok(json) => {
                if let Err(e) = fs::write(&file_path, json) {
                    eprintln!("Failed to write services info to file: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to serialize services info: {}", e),
        }
    }
}

pub fn load_services_info() -> HashMap<String, ServiceInfo> {
    if let Some(file_path) = services_json_path() {
        if file_path.exists() {
            match fs::read_to_string(&file_path) {
                Ok(content) => match serde_json::from_str::<HashMap<String, ServiceInfo>>(&content) {
                    Ok(data) => return data,
                    Err(e) => eprintln!("Failed to parse services info JSON: {}", e),
                },
                Err(e) => eprintln!("Failed to read services info file: {}", e),
            }
        }
    }

    HashMap::new()
}
