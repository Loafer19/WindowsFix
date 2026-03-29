use std::collections::HashMap;

use windows::core::PCWSTR;
use windows::Win32::System::Services::*;

use crate::models::{ServiceInfo, WindowsService};
use crate::windows_api::{to_wide_string, ScHandle};

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
    eprintln!("DEBUG: Starting get_windows_services");
    unsafe {
        eprintln!("DEBUG: Opening SCM");
        let scm = OpenSCManagerW(None, None, SC_MANAGER_ENUMERATE_SERVICE)
            .map_err(|e| format!("Failed to open SCM: {:?}", e))?;
        let _scm_guard = ScHandle::new(scm);
        eprintln!("DEBUG: SCM opened successfully");

        let mut bytes_needed: u32 = 0;
        let mut services_returned: u32 = 0;

        eprintln!("DEBUG: Calling EnumServicesStatusExW to get buffer size");
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
                return Err(format!("Failed to get buffer size: {:?}", e));
            }
        }

        eprintln!("DEBUG: Initial bytes_needed: {}", bytes_needed);
        if bytes_needed == 0 {
            eprintln!("DEBUG: No services found");
            return Ok(vec![]);
        }

        let mut buffer = vec![0u8; bytes_needed as usize];
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 10;

        eprintln!("DEBUG: Starting enumeration loop");
        loop {
            attempts += 1;
            eprintln!("DEBUG: Attempt {} of {}", attempts, MAX_ATTEMPTS);
            if attempts > MAX_ATTEMPTS {
                eprintln!("DEBUG: Too many attempts");
                return Err("Too many attempts to enumerate services".to_string());
            }

            eprintln!("DEBUG: Calling EnumServicesStatusExW with buffer");
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
                    eprintln!("DEBUG: Buffer too small, resizing to {}", bytes_needed * 2);
                    bytes_needed *= 2;
                    buffer.resize(bytes_needed as usize, 0);
                    continue;
                } else {
                    eprintln!("DEBUG: Failed to enumerate services: {:?}", e);
                    return Err(format!("Failed to enumerate services: {:?}", e));
                }
            } else {
                eprintln!("DEBUG: Enumeration successful, services_returned: {}", services_returned);
                break;
            }
        }

        let service_infos = std::slice::from_raw_parts(
            buffer.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW,
            services_returned as usize,
        );

        eprintln!("DEBUG: Processing {} services", service_infos.len());
        let mut services = Vec::new();

        for (i, service_info) in service_infos.iter().enumerate() {
            if i % 50 == 0 {
                eprintln!("DEBUG: Processing service {} of {}", i + 1, service_infos.len());
            }
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
                let st = query_startup_type(service);
                CloseServiceHandle(service).ok();
                st
            } else {
                eprintln!("DEBUG: Failed to open service {} for startup type", name);
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

        eprintln!("DEBUG: Finished processing all services, total: {}", services.len());
        Ok(services)
    }
}

/// Common helper: open the SCM and the named service, run `f`, then query and return the updated state.
fn apply_service_operation<F>(
    service_name: &str,
    access: u32,
    f: F,
) -> Result<WindowsService, String>
where
    F: FnOnce(SC_HANDLE) -> Result<(), String>,
{
    unsafe {
        let scm = OpenSCManagerW(None, None, SC_MANAGER_CONNECT)
            .map_err(|e| format!("Failed to open SCM: {:?}", e))?;
        let _scm_guard = ScHandle::new(scm);

        let name_wide = to_wide_string(service_name);
        let service = OpenServiceW(scm, PCWSTR::from_raw(name_wide.as_ptr()), access)
            .map_err(|e| format!("Failed to open service '{}': {:?}", service_name, e))?;
        let service_guard = ScHandle::new(service);

        f(service_guard.get())?;

        let mut status = SERVICE_STATUS::default();
        let status_str = if QueryServiceStatus(service_guard.get(), &mut status).is_ok() {
            service_status_str(status.dwCurrentState).to_string()
        } else {
            "Unknown".to_string()
        };

        let startup_type = query_startup_type(service_guard.get());

        Ok(WindowsService {
            name: service_name.to_string(),
            display_name: service_name.to_string(),
            status: status_str,
            startup_type,
            info: ServiceInfo { description: None, explained: None, recommendation: None },
        })
    }
}

pub fn start_windows_service(service_name: &str) -> Result<WindowsService, String> {
    apply_service_operation(
        service_name,
        SERVICE_START | SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG,
        |service| unsafe {
            StartServiceW(service, None)
                .map_err(|e| format!("Failed to start service: {:?}", e))
        },
    )
}

pub fn stop_windows_service(service_name: &str) -> Result<WindowsService, String> {
    apply_service_operation(
        service_name,
        SERVICE_STOP | SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG,
        |service| unsafe {
            let mut status = SERVICE_STATUS::default();
            ControlService(service, SERVICE_CONTROL_STOP, &mut status)
                .map_err(|e| format!("Failed to stop service: {:?}", e))
        },
    )
}

pub fn restart_windows_service(service_name: &str) -> Result<WindowsService, String> {
    apply_service_operation(
        service_name,
        SERVICE_START | SERVICE_STOP | SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG,
        |service| unsafe {
            // Stop if currently running; log but don't fail so we can still start
            let mut status = SERVICE_STATUS::default();
            if QueryServiceStatus(service, &mut status).is_ok()
                && status.dwCurrentState == SERVICE_RUNNING
            {
                if let Err(e) = ControlService(service, SERVICE_CONTROL_STOP, &mut status) {
                    eprintln!("Warning: failed to stop service before restart: {:?}", e);
                }
                // Give the SCM time to begin stopping
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            StartServiceW(service, None)
                .map_err(|e| format!("Failed to start service: {:?}", e))
        },
    )
}

pub fn set_windows_service_startup_type(
    service_name: &str,
    startup_type: &str,
) -> Result<WindowsService, String> {
    let start_type = match startup_type {
        "Automatic" => SERVICE_AUTO_START,
        "Manual" => SERVICE_DEMAND_START,
        "Disabled" => SERVICE_DISABLED,
        "Boot" => SERVICE_BOOT_START,
        "System" => SERVICE_SYSTEM_START,
        _ => return Err(format!("Unknown startup type: {}", startup_type)),
    };

    apply_service_operation(
        service_name,
        SERVICE_CHANGE_CONFIG | SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG,
        move |service| unsafe {
            // Query existing config to preserve service type and error control
            let (service_type, error_control) = read_service_type_and_error_control(service);
            ChangeServiceConfigW(
                service,
                ENUM_SERVICE_TYPE(service_type.0),
                start_type,
                SERVICE_ERROR(error_control.0),
                None, None, None, None, None, None, None,
            )
            .map_err(|e| format!("Failed to change startup type: {:?}", e))
        },
    )
}

pub fn disable_windows_service(service_name: &str) -> Result<WindowsService, String> {
    apply_service_operation(
        service_name,
        SERVICE_CHANGE_CONFIG | SERVICE_STOP | SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG,
        |service| unsafe {
            // Stop if currently running
            let mut status = SERVICE_STATUS::default();
            if QueryServiceStatus(service, &mut status).is_ok()
                && status.dwCurrentState == SERVICE_RUNNING
            {
                ControlService(service, SERVICE_CONTROL_STOP, &mut status).ok();
            }
            // Preserve service type and error control, only change start type
            let (service_type, error_control) = read_service_type_and_error_control(service);
            ChangeServiceConfigW(
                service,
                ENUM_SERVICE_TYPE(service_type.0),
                SERVICE_DISABLED,
                SERVICE_ERROR(error_control.0),
                None, None, None, None, None, None, None,
            )
            .map_err(|e| format!("Failed to disable service: {:?}", e))
        },
    )
}

/// Query the current service type and error control, falling back to safe defaults.
unsafe fn read_service_type_and_error_control(
    service: SC_HANDLE,
) -> (ENUM_SERVICE_TYPE, SERVICE_ERROR) {
    let mut config_size: u32 = 0;
    let _ = QueryServiceConfigW(service, None, 0, &mut config_size);
    if config_size > 0 {
        let mut buf = vec![0u8; config_size as usize];
        if QueryServiceConfigW(
            service,
            Some(buf.as_mut_ptr() as *mut _),
            config_size,
            &mut config_size,
        )
        .is_ok()
        {
            let cfg = &*(buf.as_ptr() as *const QUERY_SERVICE_CONFIGW);
            return (cfg.dwServiceType, cfg.dwErrorControl);
        }
    }
    (SERVICE_WIN32_OWN_PROCESS, SERVICE_ERROR_NORMAL)
}

fn query_startup_type(service: SC_HANDLE) -> String {
    unsafe {
        let mut config_size: u32 = 0;
        let _ = QueryServiceConfigW(service, None, 0, &mut config_size);
        if config_size == 0 {
            return "Unknown".to_string();
        }
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
            startup_type_str(config.dwStartType).to_string()
        } else {
            "Unknown".to_string()
        }
    }
}
