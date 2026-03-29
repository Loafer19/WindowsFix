use crate::models::WinDivertStatus;

// Embedded WinDivert driver file — included at compile time.
static WINDRIVER_SYS: &[u8] = include_bytes!("../../drivers/WinDivert64.sys");

#[tauri::command]
pub async fn check_windivert_status() -> Result<WinDivertStatus, String> {
    use std::path::Path;
    use windows::Win32::System::Services::*;

    let library_exists =
        Path::new(r"C:\Windows\System32\drivers\WinDivert64.sys").exists();

    let mut service_exists = false;
    let mut service_running = false;

    unsafe {
        let scm = match OpenSCManagerW(None, None, SC_MANAGER_CONNECT) {
            Ok(h) => h,
            Err(_) => {
                return Ok(WinDivertStatus {
                    library_exists,
                    service_exists: false,
                    service_running: false,
                })
            }
        };

        let service_name = "WinDivert"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        if let Ok(service) = OpenServiceW(
            scm,
            windows::core::PCWSTR::from_raw(service_name.as_ptr()),
            SERVICE_QUERY_STATUS,
        ) {
            let mut status = SERVICE_STATUS::default();
            if QueryServiceStatus(service, &mut status).is_ok() {
                service_exists = true;
                service_running = status.dwCurrentState == SERVICE_RUNNING;
            }
            CloseServiceHandle(service).ok();
        }

        CloseServiceHandle(scm).ok();
    }

    Ok(WinDivertStatus {
        library_exists,
        service_exists,
        service_running,
    })
}

#[tauri::command]
pub async fn install_windivert() -> Result<(), String> {
    use std::fs;
    use std::path::Path;
    use windows::Win32::System::Services::*;

    // Write embedded WinDivert driver to system location
    let drivers_path = Path::new(r"C:\Windows\System32\drivers\WinDivert64.sys");
    fs::write(drivers_path, WINDRIVER_SYS)
        .map_err(|e| format!("Failed to write WinDivert64.sys to drivers: {}", e))?;

    // Create service using Windows APIs
    unsafe {
        let scm = OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS)
            .map_err(|e| format!("Failed to open SCM: {:?}", e))?;

        let service_name = "WinDivert"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        let display_name = "WinDivert Packet Divert Service"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        let bin_path = r"\SystemRoot\System32\drivers\WinDivert64.sys"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();

        // Delete existing service if it exists
        if let Ok(existing_service) = OpenServiceW(
            scm,
            windows::core::PCWSTR::from_raw(service_name.as_ptr()),
            SERVICE_ALL_ACCESS,
        ) {
            let mut status = SERVICE_STATUS::default();
            if QueryServiceStatus(existing_service, &mut status).is_ok()
                && status.dwCurrentState == SERVICE_RUNNING
            {
                ControlService(existing_service, SERVICE_CONTROL_STOP, &mut status).ok();
            }
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
        )
        .map_err(|e| format!("Failed to create service: {:?}", e))?;

        // Start service
        StartServiceW(service, Some(&[]))
            .map_err(|e| format!("Failed to start service: {:?}", e))?;

        CloseServiceHandle(service).ok();
        CloseServiceHandle(scm).ok();
    }

    Ok(())
}

#[tauri::command]
pub async fn start_windivert_service() -> Result<(), String> {
    use std::path::Path;
    use std::process::Command;

    // First check if the driver file exists
    if !Path::new(r"C:\Windows\System32\drivers\WinDivert64.sys").exists() {
        return Err(
            "WinDivert64.sys not found in drivers folder. Please reinstall WinDivert.".to_string(),
        );
    }

    // Start service using sc.exe
    let output = Command::new("sc")
        .args(["start", "WinDivert"])
        .output()
        .map_err(|e| format!("Failed to run sc start: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "sc start failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn exit_app(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}
