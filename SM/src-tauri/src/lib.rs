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

// Service information database for common Windows services
fn get_default_service_info(service_name: &str) -> ServiceInfo {
    let defaults: HashMap<&str, ServiceInfo> = [
        ("wuauserv", ServiceInfo {
            url: None,
            description: Some("Windows Update service that enables the detection, download, and installation of updates for Windows and other programs.".to_string()),
            explained: Some("The Windows Update service (wuauserv) is responsible for managing updates to the Windows operating system and installed applications. It checks for available updates from Microsoft servers, downloads them, and coordinates their installation. This service is essential for keeping the system secure and up-to-date with the latest patches and improvements.".to_string()),
            recommendation: Some("Keep this service running on Automatic startup. Windows Update is critical for system security. Only disable temporarily if experiencing update-related issues, but re-enable as soon as possible.".to_string()),
            error: Some(false),
            message: None,
        }),
        ("spooler", ServiceInfo {
            url: None,
            description: Some("Print Spooler service that manages print jobs and printer drivers.".to_string()),
            explained: Some("The Print Spooler service manages all print jobs sent to printers connected to the computer. It acts as a queue for print jobs, allowing multiple documents to be printed in sequence. The service also manages printer drivers and handles communication between applications and printers.".to_string()),
            recommendation: Some("Set to Manual startup if you don't use printers regularly. This service only needs to run when printing. If you have network printers or frequently print documents, keep it on Automatic.".to_string()),
            error: Some(false),
            message: None,
        }),
        ("audiosrv", ServiceInfo {
            url: None,
            description: Some("Windows Audio service that manages audio devices and system sounds.".to_string()),
            explained: Some("The Windows Audio service manages all audio devices connected to the system, including speakers, microphones, and other audio hardware. It handles audio playback, recording, and system sound effects. This service is essential for any audio functionality on Windows.".to_string()),
            recommendation: Some("Keep on Automatic startup. This service is required for all audio functionality. Only disable if you have no audio devices or are troubleshooting audio issues.".to_string()),
            error: Some(false),
            message: None,
        }),
        ("eventlog", ServiceInfo {
            url: None,
            description: Some("Windows Event Log service that records system and application events.".to_string()),
            explained: Some("The Event Log service collects and stores event messages from Windows components and applications. These logs are crucial for troubleshooting system issues, monitoring system health, and auditing system activity. Event logs contain information about system errors, warnings, and informational messages.".to_string()),
            recommendation: Some("Keep on Automatic startup. Event logging is essential for system diagnostics and troubleshooting. This service should never be disabled as it provides critical system monitoring capabilities.".to_string()),
            error: Some(false),
            message: None,
        }),
        ("dnscache", ServiceInfo {
            url: None,
            description: Some("DNS Client service that resolves domain names to IP addresses.".to_string()),
            explained: Some("The DNS Client service caches Domain Name System (DNS) name resolutions. When you visit a website, this service translates human-readable domain names (like google.com) into IP addresses that computers use to communicate. The cache improves performance by storing recent DNS lookups.".to_string()),
            recommendation: Some("Keep on Automatic startup. DNS resolution is fundamental to internet connectivity. Only disable for specific troubleshooting scenarios, but re-enable immediately after.".to_string()),
            error: Some(false),
            message: None,
        }),
        ("themes", ServiceInfo {
            url: None,
            description: Some("Themes service that manages desktop themes and visual styles.".to_string()),
            explained: Some("The Themes service manages the visual appearance of Windows, including desktop wallpapers, window colors, sounds, and screen savers. It handles theme switching and ensures consistent visual styling across the operating system.".to_string()),
            recommendation: Some("Can be set to Manual startup if you don't use themes or visual customizations. However, keeping it on Automatic provides better visual consistency and theme support.".to_string()),
            error: Some(false),
            message: None,
        }),
        ("sysmain", ServiceInfo {
            url: None,
            description: Some("SysMain service that maintains and improves system performance.".to_string()),
            explained: Some("SysMain (formerly SuperFetch) analyzes system usage patterns and preloads frequently used applications and data into memory. This service helps improve system responsiveness by anticipating user actions and preparing resources in advance.".to_string()),
            recommendation: Some("Keep on Automatic for better system performance. This service helps Windows run more smoothly by optimizing memory usage. Only disable if you suspect it's causing performance issues.".to_string()),
            error: Some(false),
            message: None,
        }),
        ("wscsvc", ServiceInfo {
            url: None,
            description: Some("Security Center service that monitors security settings and health.".to_string()),
            explained: Some("The Security Center service monitors the status of Windows Security features including antivirus software, firewall, automatic updates, and other security settings. It provides notifications about security issues and helps maintain overall system security posture.".to_string()),
            recommendation: Some("Keep on Automatic startup. Security monitoring is crucial for system protection. This service helps ensure your security software is functioning properly.".to_string()),
            error: Some(false),
            message: None,
        }),
        ("bits", ServiceInfo {
            url: None,
            description: Some("Background Intelligent Transfer Service for efficient data transfers.".to_string()),
            explained: Some("BITS transfers files between clients and servers in the background using idle network bandwidth. It's used by Windows Update, Microsoft Store, and other Microsoft services for downloading updates and content without interfering with foreground network activity.".to_string()),
            recommendation: Some("Keep on Manual startup. BITS only runs when needed for background transfers. This is the recommended setting to conserve system resources while maintaining functionality.".to_string()),
            error: Some(false),
            message: None,
        }),
        ("cryptsvc", ServiceInfo {
            url: None,
            description: Some("Cryptographic Services that provides cryptographic operations.".to_string()),
            explained: Some("The Cryptographic Services provides three management services: Catalog Database Service, Protected Root Service, and Key Service. These services support cryptographic operations and are required for Windows Update, Microsoft Store, and other system functions that require digital signatures or encryption.".to_string()),
            recommendation: Some("Keep on Automatic startup. Cryptographic services are essential for system security and many Windows features. Only disable for specific troubleshooting scenarios.".to_string()),
            error: Some(false),
            message: None,
        }),
    ].into_iter().collect();

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
        url: None,
        description: Some(format!("Windows service: {}", service_name)),
        explained: Some(format!("This is a Windows system service named '{}'. It performs specific functions within the Windows operating system. For detailed information about this service, please check Microsoft's official documentation or use the reload function to get AI-generated explanations.", service_name)),
        recommendation: Some("Before making changes to this service, research its specific function. Many Windows services are essential for system stability. Consider the impact on system functionality before disabling any service.".to_string()),
        error: Some(false),
        message: None,
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
    pub url: Option<String>,
    pub description: Option<String>,
    pub explained: Option<String>,
    pub recommendation: Option<String>,
    pub error: Option<bool>,
    pub message: Option<String>,
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
            save_services_info(&services_info);

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
    match disable_windows_service(&service_name).await {
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
                    url: None,
                    description: None,
                    explained: None,
                    recommendation: None,
                    error: Some(true),
                    message: Some("Not loaded".to_string()),
                },
            });
        }

        CloseServiceHandle(scm).ok();
        Ok(services)
    }
}

async fn disable_windows_service(service_name: &str) -> Result<WindowsService, String> {
    // Use Windows API to disable service
    // This is a placeholder - implement actual Windows service control

    Ok(WindowsService {
        name: service_name.to_string(),
        display_name: format!("{} (Disabled)", service_name),
        status: "Stopped".to_string(),
        startup_type: "Disabled".to_string(),
        info: ServiceInfo {
            url: None,
            description: None,
            explained: None,
            recommendation: None,
            error: Some(false),
            message: None,
        },
    })
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

    let prompt = format!("What is the Windows service \"{}\"?\n\nPlease provide:\n1. A brief description of what this service does\n2. A detailed explanation of its purpose and functionality\n3. A recommendation on whether users should disable it and why\n\nFormat your response as JSON with keys: \"description\", \"explained\", \"recommendation\"", service_name);

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
            url: None,
            description: parsed["description"].as_str().map(|s| s.to_string()),
            explained: parsed["explained"].as_str().map(|s| s.to_string()),
            recommendation: parsed["recommendation"].as_str().map(|s| s.to_string()),
            error: Some(false),
            message: None,
        })
    } else {
        // Fallback: extract information from text response
        println!("AI response not valid JSON, trying text extraction");
        let description = extract_field_from_text(ai_response, "description");
        let explained = extract_field_from_text(ai_response, "explained");
        let recommendation = extract_field_from_text(ai_response, "recommendation");

        Ok(ServiceInfo {
            url: None,
            description: description.or_else(|| Some("AI-generated description".to_string())),
            explained: explained.or_else(|| Some("AI-generated explanation".to_string())),
            recommendation: recommendation.or_else(|| Some("AI-generated recommendation".to_string())),
            error: Some(false),
            message: None,
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
    if let Some(app_dir) = directories::ProjectDirs::from("com", "servicesmanager", "app") {
        let data_dir = app_dir.data_dir();
        match fs::create_dir_all(data_dir) {
            Ok(_) => {
                let file_path = data_dir.join("services-info.json");
                match serde_json::to_string_pretty(services_info) {
                    Ok(json) => {
                        if let Err(e) = fs::write(&file_path, json) {
                            eprintln!("Failed to write services info to file: {}", e);
                        } else {
                            println!("Successfully saved {} service info entries", services_info.len());
                        }
                    },
                    Err(e) => eprintln!("Failed to serialize services info: {}", e),
                }
            },
            Err(e) => eprintln!("Failed to create data directory: {}", e),
        }
    } else {
        eprintln!("Failed to get application directory for data storage");
    }
}

fn load_services_info() -> HashMap<String, ServiceInfo> {
    if let Some(app_dir) = directories::ProjectDirs::from("com", "servicesmanager", "app") {
        let data_dir = app_dir.data_dir();
        let file_path = data_dir.join("services-info.json");

        // Try to load from user data directory first
        if file_path.exists() {
            match fs::read_to_string(&file_path) {
                Ok(content) => {
                    match serde_json::from_str::<HashMap<String, ServiceInfo>>(&content) {
                        Ok(data) => {
                            println!("Loaded {} service info entries from user data", data.len());
                            return data;
                        },
                        Err(e) => eprintln!("Failed to parse services info JSON: {}", e),
                    }
                },
                Err(e) => eprintln!("Failed to read services info file: {}", e),
            }
        }

        // Try to load from bundled location as fallback
        if let Ok(current_dir) = std::env::current_dir() {
            let bundled_path = current_dir.join("ServicesManager/server/public/services-info.json");
            if bundled_path.exists() {
                match fs::read_to_string(&bundled_path) {
                    Ok(content) => {
                        match serde_json::from_str::<HashMap<String, ServiceInfo>>(&content) {
                            Ok(data) => {
                                println!("Loaded {} service info entries from bundled data", data.len());
                                // Save to user data directory for future use
                                if fs::create_dir_all(data_dir).is_ok() {
                                    if let Ok(json) = serde_json::to_string_pretty(&data) {
                                        if let Err(e) = fs::write(&file_path, json) {
                                            eprintln!("Failed to save bundled data to user directory: {}", e);
                                        } else {
                                            println!("Saved bundled data to user directory");
                                        }
                                    }
                                }
                                return data;
                            },
                            Err(e) => eprintln!("Failed to parse bundled services info JSON: {}", e),
                        }
                    },
                    Err(e) => eprintln!("Failed to read bundled services info file: {}", e),
                }
            }
        }
    } else {
        eprintln!("Failed to get application directory for data loading");
    }

    println!("No existing service info found, starting with empty database");
    HashMap::new()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load environment variables from .env file
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
