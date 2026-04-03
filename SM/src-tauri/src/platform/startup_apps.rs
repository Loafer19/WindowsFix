use std::path::PathBuf;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
use windows::Win32::System::Registry::*;

use super::windows_api::to_wide_string;
use crate::core::models::{AppError, StartupApp, StartupLocation};

const RUN_KEY: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";

/// Maximum character count for a registry value name buffer (including null terminator).
const MAX_REGISTRY_NAME_SIZE: usize = 256;
/// Maximum byte size for a registry value data buffer.
const MAX_REGISTRY_VALUE_SIZE: usize = 2048;

/// List all startup entries from HKCU, HKLM, and the user startup folder.
pub fn list_startup_apps() -> Result<Vec<StartupApp>, AppError> {
    let mut apps = Vec::new();
    apps.extend(read_registry_startup(
        HKEY_CURRENT_USER,
        StartupLocation::HkeyCurrentUser,
    )?);
    apps.extend(read_registry_startup(
        HKEY_LOCAL_MACHINE,
        StartupLocation::HkeyLocalMachine,
    )?);
    apps.extend(read_startup_folder()?);
    Ok(apps)
}

fn read_registry_startup(
    hive: HKEY,
    location: StartupLocation,
) -> Result<Vec<StartupApp>, AppError> {
    let run_wide = to_wide_string(RUN_KEY);
    let mut apps = Vec::new();

    unsafe {
        let mut key = HKEY::default();
        let result = RegOpenKeyExW(
            hive,
            PCWSTR::from_raw(run_wide.as_ptr()),
            0,
            KEY_READ,
            &mut key,
        );
        if result != ERROR_SUCCESS {
            // Key may not exist or access may be denied; treat as empty
            return Ok(vec![]);
        }

        let mut index: u32 = 0;
        loop {
            let mut name_buf = vec![0u16; MAX_REGISTRY_NAME_SIZE];
            let mut name_len = name_buf.len() as u32;
            let mut value_type: u32 = 0;
            let mut data_buf = vec![0u8; MAX_REGISTRY_VALUE_SIZE];
            let mut data_len = data_buf.len() as u32;

            let result = RegEnumValueW(
                key,
                index,
                PWSTR(name_buf.as_mut_ptr()),
                &mut name_len,
                None,
                Some(&mut value_type),
                Some(data_buf.as_mut_ptr()),
                Some(&mut data_len),
            );

            if result == ERROR_NO_MORE_ITEMS {
                break;
            }
            if result != ERROR_SUCCESS {
                index += 1;
                continue;
            }

            let name = String::from_utf16_lossy(&name_buf[..name_len as usize]).to_string();

            // Only handle REG_SZ and REG_EXPAND_SZ
            if value_type != 1 && value_type != 2 {
                index += 1;
                continue;
            }

            // Data is UTF-16 LE, possibly null-terminated
            let word_count = data_len as usize / 2;
            let words: Vec<u16> = data_buf[..data_len as usize]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let end = words[..word_count]
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(word_count);
            let command = String::from_utf16_lossy(&words[..end]).to_string();

            // Store full command as path, no parsing
            let path = command;
            let arguments = None;

            apps.push(StartupApp {
                name,
                path,
                arguments,
                location: location.clone(),
                enabled: true,
                description: None,
            });

            index += 1;
        }

        let _ = RegCloseKey(key).ok();
    }

    Ok(apps)
}

fn read_startup_folder() -> Result<Vec<StartupApp>, AppError> {
    let startup_dir = startup_folder_path();
    let mut apps = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&startup_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .map(|e| e.eq_ignore_ascii_case("lnk"))
                .unwrap_or(false)
            {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                apps.push(StartupApp {
                    name,
                    path: path.to_string_lossy().to_string(),
                    arguments: None,
                    location: StartupLocation::StartupFolder,
                    enabled: true,
                    description: None,
                });
            }
        }
    }

    Ok(apps)
}

fn startup_folder_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
}

/// Remove a startup entry from the registry or startup folder.
pub fn remove_startup_app(name: &str, location: &StartupLocation) -> Result<(), AppError> {
    match location {
        StartupLocation::HkeyCurrentUser => remove_registry_entry(HKEY_CURRENT_USER, name),
        StartupLocation::HkeyLocalMachine => remove_registry_entry(HKEY_LOCAL_MACHINE, name),
        StartupLocation::StartupFolder => remove_folder_entry(name),
    }
}

fn remove_registry_entry(hive: HKEY, name: &str) -> Result<(), AppError> {
    let run_wide = to_wide_string(RUN_KEY);
    let name_wide = to_wide_string(name);

    unsafe {
        let mut key = HKEY::default();
        RegOpenKeyExW(
            hive,
            PCWSTR::from_raw(run_wide.as_ptr()),
            0,
            KEY_WRITE,
            &mut key,
        )
        .ok()
        .map_err(|e| AppError::WindowsApi {
            message: format!("Failed to open registry key: {:?}", e),
        })?;

        let result = RegDeleteValueW(key, PCWSTR::from_raw(name_wide.as_ptr()))
            .ok()
            .map_err(|e| AppError::WindowsApi {
                message: format!("Failed to delete registry value '{}': {:?}", name, e),
            });

        let _ = RegCloseKey(key).ok();
        result
    }
}

fn remove_folder_entry(name: &str) -> Result<(), AppError> {
    let path = startup_folder_path().join(format!("{}.lnk", name));
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| AppError::Io {
            message: format!("Failed to remove startup entry '{}': {}", name, e),
        })
    } else {
        Err(AppError::Validation {
            message: format!("Startup entry '{}' not found in the startup folder", name),
        })
    }
}

/// Add a startup registry entry (HKCU or HKLM only; startup folder not supported).
pub fn add_startup_app(app: &StartupApp) -> Result<(), AppError> {
    let command = if let Some(args) = &app.arguments {
        format!("{} {}", app.path, args)
    } else {
        app.path.clone()
    };

    match &app.location {
        StartupLocation::HkeyCurrentUser => {
            add_registry_entry(HKEY_CURRENT_USER, &app.name, &command)
        }
        StartupLocation::HkeyLocalMachine => {
            add_registry_entry(HKEY_LOCAL_MACHINE, &app.name, &command)
        }
        StartupLocation::StartupFolder => Err(AppError::Validation {
            message: "Adding entries to the startup folder is not supported via this interface"
                .to_string(),
        }),
    }
}

fn add_registry_entry(hive: HKEY, name: &str, command: &str) -> Result<(), AppError> {
    let run_wide = to_wide_string(RUN_KEY);
    let name_wide = to_wide_string(name);
    let command_wide = to_wide_string(command);

    unsafe {
        let mut key = HKEY::default();
        RegOpenKeyExW(
            hive,
            PCWSTR::from_raw(run_wide.as_ptr()),
            0,
            KEY_WRITE,
            &mut key,
        )
        .ok()
        .map_err(|e| AppError::WindowsApi {
            message: format!("Failed to open registry key: {:?}", e),
        })?;

        // command_wide includes the null terminator; pass the full byte slice
        let data =
            std::slice::from_raw_parts(command_wide.as_ptr() as *const u8, command_wide.len() * 2);
        let result = RegSetValueExW(
            key,
            PCWSTR::from_raw(name_wide.as_ptr()),
            0,
            REG_SZ,
            Some(data),
        )
        .ok()
        .map_err(|e| AppError::WindowsApi {
            message: format!("Failed to set registry value '{}': {:?}", name, e),
        });

        let _ = RegCloseKey(key).ok();
        result
    }
}
