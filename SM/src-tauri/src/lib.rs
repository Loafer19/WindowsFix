// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::time::timeout;
use windows::core::PCWSTR;
use windows::Win32::System::Services::*;

fn load_services() -> HashMap<String, ServiceInfo> {
    // Try to load from services.json relative to src-tauri directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(src_tauri_dir) = exe_path.parent().and_then(|p| p.parent()) {
            let json_path = src_tauri_dir.join("services.json");
            if let Ok(content) = fs::read_to_string(json_path) {
                if let Ok(data) = serde_json::from_str::<HashMap<String, ServiceInfo>>(&content) {
                    println!("Loaded {} services from services.json", data.len());
                    return data;
                }
            }
        }
    }

    // Fallback to minimal hardcoded defaults if JSON loading fails
    println!("Failed to load services from JSON, using minimal fallback");
    let mut defaults = HashMap::new();
    defaults.insert("wuauserv".to_string(), ServiceInfo {
        description: Some("Windows Update service".to_string()),
        explained: Some("Manages Windows updates and security patches.".to_string()),
        recommendation: Some("• Keep Automatic for security\n• Critical system service".to_string()),
    });
    defaults
}

// Service information database for common Windows services
fn get_default_service_info(service_name: &str) -> ServiceInfo {
    let defaults = load_services();

    // Check for exact match first
    if let Some(info) = defaults.get(service_name) {
        return info.clone();
    }

    // Check for partial matches (case-insensitive)
    let service_lower = service_name.to_lowercase();
    for (key, info) in &defaults {
        if service_lower.contains(&key.to_lowercase()) || key.to_lowercase().contains(&service_lower) {
            return info.clone();
        }
    }

    // Return generic info if no match found
    ServiceInfo {
        description: Some(format!("Windows service: {}", service_name)),
        explained: Some(format!("Windows system service '{}'. Performs specific OS functions. Use reload for AI-generated detailed explanation.", service_name)),
        recommendation: Some("• Research service function before changes\n• Many services are essential for stability\n• Consider functionality impact before disabling\n• Use reload button for detailed AI analysis".to_string()),
    }
}

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsService {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub status: String,
    #[serde(rename = "startupType")]
    pub startup_type: String,
    pub info: ServiceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub description: Option<String>,
    pub explained: Option<String>,
    pub recommendation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServicesResponse {
    pub services: Vec<WindowsService>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub service: WindowsService,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshResponse {
    pub message: String,
    pub count: usize,
    pub timestamp: String,
}

// App state
pub struct AppState {
    pub services_cache: Mutex<ServicesCache>,
    pub services_info: Mutex<HashMap<String, ServiceInfo>>,
}

pub struct ServicesCache {
    pub data: Vec<WindowsService>,
    pub last_updated: SystemTime,
    pub ttl: Duration,
}

// Tauri commands
#[tauri::command]
async fn get_services(state: State<'_, AppState>) -> Result<ServicesResponse, String> {
    let needs_refresh = {
        let cache = state.services_cache.lock().unwrap();
        cache.data.is_empty() || SystemTime::now().duration_since(cache.last_updated).unwrap_or(Duration::MAX) > cache.ttl
    };

    if needs_refresh {
        // Refresh cache
        match refresh_services_cache(&state.services_info).await {
            Ok(new_data) => {
                let mut cache = state.services_cache.lock().unwrap();
                cache.data = new_data;
                cache.last_updated = SystemTime::now();
            },
            Err(e) => return Err(format!("Failed to refresh services cache: {}", e)),
        }
    }

    let cache = state.services_cache.lock().unwrap();
    Ok(ServicesResponse {
        services: cache.data.clone(),
    })
}

#[tauri::command]
async fn refresh_services(state: State<'_, AppState>) -> Result<RefreshResponse, String> {
    match refresh_services_cache(&state.services_info).await {
        Ok(new_data) => {
            let mut cache = state.services_cache.lock().unwrap();
            let count = new_data.len();
            cache.data = new_data;
            cache.last_updated = SystemTime::now();
            Ok(RefreshResponse {
                message: "Services cache refreshed".to_string(),
                count,
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
        },
        Err(e) => Err(format!("Failed to refresh services cache: {}", e)),
    }
}

#[tauri::command]
async fn reload_service_info(service_name: String, state: State<'_, AppState>) -> Result<ServiceInfo, String> {
    // Always attempt to fetch fresh information for reload
    // This ensures we get the latest data from web/AI sources
    match fetch_service_info(&service_name).await {
        Ok(info) => {
            // Update the cache with new info
            let mut cache = state.services_cache.lock().unwrap();
            if let Some(service) = cache.data.iter_mut().find(|s| s.name == service_name) {
                service.info = info.clone();
            }

            // Update persistent storage
            let mut services_info = state.services_info.lock().unwrap();
            services_info.insert(service_name.clone(), info.clone());
            // Note: Not saving to JSON on reload to avoid frequent writes

            println!("Successfully reloaded info for service: {}", service_name);
            Ok(info)
        },
        Err(e) => {
            // If fresh fetch fails, fall back to existing cached info if available
            let services_info = state.services_info.lock().unwrap();
            if let Some(existing_info) = services_info.get(&service_name) {
                if existing_info.explained.is_some() || existing_info.recommendation.is_some() {
                    println!("Fresh fetch failed, returning cached info for service: {}", service_name);
                    return Ok(existing_info.clone());
                }
            }

            // If no cached info available, return the error
            Err(format!("Failed to reload service info: {}", e))
        },
    }
}

#[tauri::command]
async fn disable_service(service_name: String, state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    let service_name_clone = service_name.clone();
    match tokio::task::spawn_blocking(move || disable_windows_service(&service_name_clone)).await {
        Ok(result) => match result {
            Ok(updated_service) => {
                // Update cache
                let mut cache = state.services_cache.lock().unwrap();
                if let Some(service) = cache.data.iter_mut().find(|s| s.name == service_name) {
                    service.status = updated_service.status.clone();
                    service.startup_type = updated_service.startup_type.clone();
                }

                Ok(ServiceResponse {
                    service: updated_service,
                })
            },
            Err(e) => Err(format!("Failed to disable service: {}", e)),
        },
        Err(e) => Err(format!("Task panicked: {:?}", e)),
    }
}

// Helper functions
async fn refresh_services_cache(services_info: &Mutex<HashMap<String, ServiceInfo>>) -> Result<Vec<WindowsService>, String> {
    match get_windows_services().await {
        Ok(services) => {
            let mut info_map = services_info.lock().unwrap();
            let processed_services: Vec<WindowsService> = services.into_iter()
                .map(|service| {
                    let name = service.name.split('_').next().unwrap_or(&service.name).to_string();

                    // Get existing info or create default
                    let info = if let Some(existing_info) = info_map.get(&name) {
                        // Use existing info if it has meaningful content
                        if existing_info.explained.is_some() || existing_info.recommendation.is_some() {
                            existing_info.clone()
                        } else {
                            // Upgrade to default info if existing is minimal
                            get_default_service_info(&name)
                        }
                    } else {
                        // No existing info, use defaults
                        let default_info = get_default_service_info(&name);
                        // Store in persistent storage for future use
                        info_map.insert(name.clone(), default_info.clone());
                        default_info
                    };

                    WindowsService {
                        name: name.clone(),
                        display_name: service.display_name,
                        status: service.status,
                        startup_type: service.startup_type,
                        info,
                    }
                })
                .collect();

            // Save updated info to persistent storage
            save_services_info(&info_map);

            Ok(processed_services)
        },
        Err(e) => Err(format!("Failed to get Windows services: {}", e)),
    }
}

async fn get_windows_services() -> Result<Vec<WindowsService>, String> {
    unsafe {
        let scm = OpenSCManagerW(None, None, SC_MANAGER_ENUMERATE_SERVICE).map_err(|e| format!("Failed to open SCM: {:?}", e))?;

        let mut bytes_needed: u32 = 0;
        let mut services_returned: u32 = 0;

        // First call to get buffer size
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
                if e.code().0 == 0x800700EAu32 as i32 { // ERROR_MORE_DATA
                    // Buffer too small, increase size
                    bytes_needed = bytes_needed * 2;
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

        let service_infos = std::slice::from_raw_parts(buffer.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW, services_returned as usize);

        let mut services = Vec::new();

        for service_info in service_infos {
            let name = service_info.lpServiceName.to_string().map_err(|_| "Invalid service name")?;
            let mut display_name = service_info.lpDisplayName.to_string().map_err(|_| "Invalid display name")?;
            if display_name.is_empty() {
                display_name = name.clone();
            }

            let status = match service_info.ServiceStatusProcess.dwCurrentState {
                SERVICE_RUNNING => "Running",
                SERVICE_STOPPED => "Stopped",
                SERVICE_START_PENDING => "Start Pending",
                SERVICE_STOP_PENDING => "Stop Pending",
                SERVICE_PAUSE_PENDING => "Pause Pending",
                SERVICE_PAUSED => "Paused",
                _ => "Unknown",
            }.to_string();

            // Get startup type
            let startup_type = if let Ok(service) = OpenServiceW(scm, PCWSTR::from_raw(service_info.lpServiceName.as_ptr() as *const _), SERVICE_QUERY_CONFIG) {
                let mut config_size: u32 = 0;
                let _ = QueryServiceConfigW(service, None, 0, &mut config_size);

                let startup_type_str = if config_size > 0 {
                    let mut config_buffer = vec![0u8; config_size as usize];
                    if QueryServiceConfigW(service, Some(config_buffer.as_mut_ptr() as *mut _), config_size, &mut config_size).is_ok() {
                        let config = &*(config_buffer.as_ptr() as *const QUERY_SERVICE_CONFIGW);
                        match config.dwStartType {
                            SERVICE_AUTO_START => "Automatic",
                            SERVICE_DEMAND_START => "Manual",
                            SERVICE_DISABLED => "Disabled",
                            SERVICE_BOOT_START => "Boot",
                            SERVICE_SYSTEM_START => "System",
                            _ => "Unknown",
                        }
                    } else {
                        "Unknown"
                    }
                } else {
                    "Unknown"
                };
                CloseServiceHandle(service).ok();
                startup_type_str.to_string()
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

fn disable_windows_service(service_name: &str) -> Result<WindowsService, String> {
    unsafe {
        // Open SCM
        let scm = OpenSCManagerW(None, None, SC_MANAGER_CONNECT).map_err(|e| format!("Failed to open SCM: {:?}", e))?;

        // Open the service
        let service_name_wide = service_name.to_string().encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
        let service = OpenServiceW(scm, PCWSTR::from_raw(service_name_wide.as_ptr()), SERVICE_CHANGE_CONFIG | SERVICE_STOP | SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG).map_err(|e| format!("Failed to open service: {:?}", e))?;

        // Stop the service if running
        let mut status = SERVICE_STATUS::default();
        if QueryServiceStatus(service, &mut status).is_ok() && status.dwCurrentState == SERVICE_RUNNING {
            println!("Stopping service {}", service_name);
            let stop_result = ControlService(service, SERVICE_CONTROL_STOP, &mut status);
            if stop_result.is_ok() {
                println!("Service {} stop command sent", service_name);
            } else {
                println!("Failed to send stop command to {}: {:?}", service_name, stop_result);
            }
        } else {
            println!("Service {} is not running or query failed", service_name);
        }

        // Change startup type to disabled
        println!("Disabling service {}", service_name);
        let result = ChangeServiceConfigW(
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
        );

        if result.is_ok() {
            println!("Service {} disabled successfully", service_name);
        } else {
            println!("Failed to disable service {}: {:?}", service_name, result);
        }

        result.map_err(|e| format!("Failed to disable service: {:?}", e))?;

        // Query updated status
        let mut status = SERVICE_STATUS::default();
        let status_str = if QueryServiceStatus(service, &mut status).is_ok() {
            let s = match status.dwCurrentState {
                SERVICE_RUNNING => "Running",
                SERVICE_STOPPED => "Stopped",
                SERVICE_START_PENDING => "Start Pending",
                SERVICE_STOP_PENDING => "Stop Pending",
                SERVICE_PAUSE_PENDING => "Pause Pending",
                SERVICE_PAUSED => "Paused",
                _ => "Unknown",
            }.to_string();
            println!("Service {} status: {}", service_name, s);
            s
        } else {
            println!("Failed to query status for {}", service_name);
            "Unknown".to_string()
        };

        // Query updated startup type
        let mut config_size: u32 = 0;
        let _ = QueryServiceConfigW(service, None, 0, &mut config_size);
        let mut config_buffer = vec![0u8; config_size as usize];
        let startup_type_str = if QueryServiceConfigW(service, Some(config_buffer.as_mut_ptr() as *mut _), config_size, &mut config_size).is_ok() {
            let config = &*(config_buffer.as_ptr() as *const QUERY_SERVICE_CONFIGW);
            let st = match config.dwStartType {
                SERVICE_AUTO_START => "Automatic",
                SERVICE_DEMAND_START => "Manual",
                SERVICE_DISABLED => "Disabled",
                SERVICE_BOOT_START => "Boot",
                SERVICE_SYSTEM_START => "System",
                _ => "Unknown",
            }.to_string();
            println!("Service {} startup type: {}", service_name, st);
            st
        } else {
            println!("Failed to query config for {}", service_name);
            "Unknown".to_string()
        };

        CloseServiceHandle(service).ok();
        CloseServiceHandle(scm).ok();

        // Return updated service info
        Ok(WindowsService {
            name: service_name.to_string(),
            display_name: service_name.to_string(), // Keep original display name
            status: status_str,
            startup_type: startup_type_str,
            info: ServiceInfo {
                description: None,
                explained: None,
                recommendation: None,
            },
        })
    }
}

async fn fetch_service_info(service_name: &str) -> Result<ServiceInfo, String> {
    // Try AI first
    match fetch_service_info_from_ai(service_name).await {
        Ok(info) => return Ok(info),
        Err(_) => {},
    }

    // Final fallback - use comprehensive default database
    Ok(get_default_service_info(service_name))
}

async fn fetch_service_info_from_ai(service_name: &str) -> Result<ServiceInfo, String> {
    // Get API key from environment
    let api_key = std::env::var("GROK_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        println!("Grok API key not found in environment variables");
        return Err("Grok API key not configured. Please set GROK_API_KEY in your .env file.".to_string());
    }

    println!("Using Grok API key: {}...", &api_key[..8]);

    // Get timeout from environment (default 15 seconds for AI requests)
    let timeout_secs: u64 = std::env::var("GROK_API_TIMEOUT")
        .unwrap_or_else(|_| "15".to_string())
        .parse()
        .unwrap_or(15);

    // Get max tokens from environment (default 1000)
    let max_tokens: u32 = std::env::var("GROK_MAX_TOKENS")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .unwrap_or(1000);

    let prompt = format!("What is the Windows service \"{}\"?\n\nPlease provide a JSON response with exactly these three keys:\n- \"description\": A brief description of what this service does\n- \"explained\": A concise explanation in 2-3 lines of its purpose and functionality\n- \"recommendation\": A bullet-point list covering whether to disable it, what would be affected, and safe disabling scenarios\n\nExample format:\n{{\n  \"description\": \"Brief description here\",\n  \"explained\": \"Concise explanation here\",\n  \"recommendation\": \"• Point 1\\n• Point 2\\n• Point 3\"\n}}\n\nReturn only valid JSON, no additional text.", service_name);

    println!("Making AI request for service: {}", service_name);

    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(timeout_secs),
        client
            .post("https://api.x.ai/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": "grok-3-mini",
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": max_tokens,
                "temperature": 0.7,
                "stream": false
            }))
            .send()
    ).await.map_err(|_| format!("AI API request timed out after {} seconds", timeout_secs))?
        .map_err(|e| format!("AI API request failed: {}", e))?;

    println!("AI API response status: {}", response.status());

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        println!("AI API error {}: {}", status, error_text);
        return Err(format!("AI API error {}: {}", status, error_text));
    }

    let data: serde_json::Value = response.json().await.map_err(|e| format!("Failed to parse AI response: {}", e))?;

    println!("AI response received, parsing content...");

    let ai_response = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid AI response format: missing content".to_string())?;

    println!("AI response content: {}", &ai_response[..200]);

    // Try to parse as JSON first
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(ai_response) {
        println!("Successfully parsed AI response as JSON");
        Ok(ServiceInfo {
            description: parsed["description"].as_str().map(|s| s.to_string()),
            explained: parsed["explained"].as_str().map(|s| s.to_string()),
            recommendation: parsed["recommendation"].as_str().map(|s| s.to_string()),
        })
    } else {
        // Fallback: extract information from text response
        println!("AI response not valid JSON, trying text extraction");
        let description = extract_field_from_text(ai_response, "description");
        let explained = extract_field_from_text(ai_response, "explained");
        let recommendation = extract_field_from_text(ai_response, "recommendation");

        Ok(ServiceInfo {
            description: description.or_else(|| Some("AI-generated description".to_string())),
            explained: explained.or_else(|| Some("AI-generated explanation".to_string())),
            recommendation: recommendation.or_else(|| Some("AI-generated recommendation".to_string())),
        })
    }
}

// Helper function to extract fields from text response
fn extract_field_from_text(text: &str, field: &str) -> Option<String> {
    let patterns = [
        format!("\"{}\": \"", field),
        format!("{}: ", field),
        format!("{}\n", field),
    ];

    for pattern in &patterns {
        if let Some(start) = text.find(pattern) {
            let start_pos = start + pattern.len();
            let remaining = &text[start_pos..];

            // Find the end of the field (quote, newline, or end of text)
            let end_pos = remaining.find('"')
                .or_else(|| remaining.find('\n'))
                .unwrap_or(remaining.len());

            let value = remaining[..end_pos].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn save_services_info(services_info: &HashMap<String, ServiceInfo>) {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(src_tauri_dir) = exe_path.parent().and_then(|p| p.parent()) {
            let file_path = src_tauri_dir.join("services.json");
            match serde_json::to_string_pretty(services_info) {
                Ok(json) => {
                    if let Err(e) = fs::write(&file_path, json) {
                        eprintln!("Failed to write services info to file: {}", e);
                    } else {
                        println!("Successfully saved {} service info entries to services.json", services_info.len());
                    }
                },
                Err(e) => eprintln!("Failed to serialize services info: {}", e),
            }
        } else {
            eprintln!("Failed to determine src-tauri directory for data storage");
        }
    } else {
        eprintln!("Failed to get executable path for data storage");
    }
}

fn load_services_info() -> HashMap<String, ServiceInfo> {
    // Load from services.json relative to src-tauri directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(src_tauri_dir) = exe_path.parent().and_then(|p| p.parent()) {
            let file_path = src_tauri_dir.join("services.json");
            if file_path.exists() {
                match fs::read_to_string(&file_path) {
                    Ok(content) => {
                        match serde_json::from_str::<HashMap<String, ServiceInfo>>(&content) {
                            Ok(data) => {
                                println!("Loaded {} service info entries from services.json", data.len());
                                return data;
                            },
                            Err(e) => eprintln!("Failed to parse services info JSON: {}", e),
                        }
                    },
                    Err(e) => eprintln!("Failed to read services info file: {}", e),
                }
            }
        }
    }

    println!("No existing service info found, starting with empty database");
    HashMap::new()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = dotenvy::dotenv() {
        println!("Warning: Could not load .env file: {}", e);
    } else {
        println!("Successfully loaded .env file");
    }

    let services_info = load_services_info();

    let app_state = AppState {
        services_cache: Mutex::new(ServicesCache {
            data: Vec::new(),
            last_updated: SystemTime::UNIX_EPOCH,
            ttl: Duration::from_secs(300), // 5 minutes
        }),
        services_info: Mutex::new(services_info),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_services,
            refresh_services,
            reload_service_info,
            disable_service
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
