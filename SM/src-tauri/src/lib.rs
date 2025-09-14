// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::time::timeout;

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsService {
    pub name: String,
    pub display_name: String,
    pub state: String,
    pub start_mode: String,
    pub info: ServiceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub url: Option<String>,
    pub description: Option<String>,
    pub explained: Option<String>,
    pub recommendation: Option<String>,
    pub source: Option<String>,
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
    match fetch_service_info(&service_name).await {
        Ok(info) => {
            // Update the cache with new info
            let mut cache = state.services_cache.lock().unwrap();
            if let Some(service) = cache.data.iter_mut().find(|s| s.name == service_name) {
                service.info = info.clone();
            }

            // Save to file
            save_services_info(&state.services_info.lock().unwrap());

            Ok(info)
        },
        Err(e) => Err(format!("Failed to reload service info: {}", e)),
    }
}

#[tauri::command]
async fn disable_service(service_name: String, state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    match disable_windows_service(&service_name).await {
        Ok(updated_service) => {
            // Update cache
            let mut cache = state.services_cache.lock().unwrap();
            if let Some(service) = cache.data.iter_mut().find(|s| s.name == service_name) {
                service.state = updated_service.state.clone();
                service.start_mode = updated_service.start_mode.clone();
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
            let info_map = services_info.lock().unwrap();
            let processed_services: Vec<WindowsService> = services.into_iter()
                .map(|service| {
                    let name = service.name.split('_').next().unwrap_or(&service.name).to_string();
                    let info = info_map.get(&name).cloned().unwrap_or_else(|| ServiceInfo {
                        url: None,
                        description: None,
                        explained: None,
                        recommendation: None,
                        source: None,
                        error: Some(true),
                        message: Some("Not loaded".to_string()),
                    });

                    WindowsService {
                        name: name.clone(),
                        display_name: service.display_name,
                        state: service.state,
                        start_mode: service.start_mode,
                        info,
                    }
                })
                .collect();

            Ok(processed_services)
        },
        Err(e) => Err(format!("Failed to get Windows services: {}", e)),
    }
}

async fn get_windows_services() -> Result<Vec<WindowsService>, String> {
    // Use Windows API to get services
    // This is a simplified implementation - in practice you'd use the windows crate
    // to call the Service Control Manager APIs

    // For now, return mock data - replace with actual Windows API calls
    Ok(vec![
        WindowsService {
            name: "wuauserv".to_string(),
            display_name: "Windows Update".to_string(),
            state: "Running".to_string(),
            start_mode: "Automatic".to_string(),
            info: ServiceInfo {
                url: None,
                description: None,
                explained: None,
                recommendation: None,
                source: None,
                error: Some(true),
                message: Some("Not loaded".to_string()),
            },
        },
        WindowsService {
            name: "spooler".to_string(),
            display_name: "Print Spooler".to_string(),
            state: "Stopped".to_string(),
            start_mode: "Manual".to_string(),
            info: ServiceInfo {
                url: None,
                description: None,
                explained: None,
                recommendation: None,
                source: None,
                error: Some(true),
                message: Some("Not loaded".to_string()),
            },
        },
    ])
}

async fn disable_windows_service(service_name: &str) -> Result<WindowsService, String> {
    // Use Windows API to disable service
    // This is a placeholder - implement actual Windows service control

    Ok(WindowsService {
        name: service_name.to_string(),
        display_name: format!("{} (Disabled)", service_name),
        state: "Stopped".to_string(),
        start_mode: "Disabled".to_string(),
        info: ServiceInfo {
            url: None,
            description: None,
            explained: None,
            recommendation: None,
            source: None,
            error: Some(false),
            message: None,
        },
    })
}

async fn fetch_service_info(service_name: &str) -> Result<ServiceInfo, String> {
    // Try web scraping first
    match fetch_service_info_from_web(service_name).await {
        Ok(info) => return Ok(info),
        Err(_) => {},
    }

    // Fallback to AI
    match fetch_service_info_from_ai(service_name).await {
        Ok(info) => return Ok(info),
        Err(_) => {},
    }

    // Final fallback
    Ok(ServiceInfo {
        url: None,
        description: Some(format!("Windows service: {}", service_name)),
        explained: Some(format!("This is a Windows system service named {}. Specific information about this service could not be retrieved from available sources.", service_name)),
        recommendation: Some("Unable to provide specific recommendations for this service. Please research this service carefully before making changes, as disabling system services can affect system stability.".to_string()),
        source: Some("fallback".to_string()),
        error: Some(false),
        message: None,
    })
}

async fn fetch_service_info_from_web(service_name: &str) -> Result<ServiceInfo, String> {
    let search_url = format!("https://win10tweaker.ru/?s={}", service_name);

    let client = reqwest::Client::new();
    let response = client.get(&search_url).send().await.map_err(|e| e.to_string())?;
    let html = response.text().await.map_err(|e| e.to_string())?;

    // Extract hrefs from HTML synchronously to avoid Send issues
    let hrefs = extract_service_hrefs(&html, service_name)?;

    // Now process the hrefs without holding references
    for href in hrefs {
        match fetch_service_detail(&href).await {
            Ok(info) => return Ok(info),
            Err(_) => continue,
        }
    }

    Err("Service not found on website".to_string())
}

fn extract_service_hrefs(html: &str, service_name: &str) -> Result<Vec<String>, String> {
    let document = scraper::Html::parse_document(html);
    let selector = scraper::Selector::parse(".fusion-post-grid").map_err(|e| e.to_string())?;

    let mut hrefs = Vec::new();
    for element in document.select(&selector) {
        let text = element.text().collect::<String>();
        if text.contains(&format!("Имя службы: {}", service_name)) {
            let link_selector = scraper::Selector::parse(".fusion-post-title a").map_err(|e| e.to_string())?;
            if let Some(link_element) = element.select(&link_selector).next() {
                if let Some(href) = link_element.value().attr("href") {
                    if href.contains("/twikinarium/services/") {
                        hrefs.push(href.to_string());
                    }
                }
            }
        }
    }
    Ok(hrefs)
}

async fn fetch_service_detail(url: &str) -> Result<ServiceInfo, String> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    let html = response.text().await.map_err(|e| e.to_string())?;

    let document = scraper::Html::parse_document(&html);

    let desc_selector = scraper::Selector::parse("p").map_err(|e| e.to_string())?;
    let description = document.select(&desc_selector)
        .find(|el| el.text().collect::<String>().contains("Описание по умолчанию"))
        .and_then(|el| el.next_sibling())
        .and_then(|sib| {
            match sib.value() {
                scraper::node::Node::Text(text) => Some(text.to_string()),
                _ => None,
            }
        })
        .map(|t| t.trim().to_string());

    let explained_selector = scraper::Selector::parse("p").map_err(|e| e.to_string())?;
    let explained = document.select(&explained_selector)
        .find(|el| el.text().collect::<String>().contains("Нормальное описание"))
        .and_then(|el| el.next_sibling())
        .and_then(|sib| {
            match sib.value() {
                scraper::node::Node::Text(text) => Some(text.to_string()),
                _ => None,
            }
        })
        .map(|t| t.trim().to_string());

    let rec_selector = scraper::Selector::parse("p").map_err(|e| e.to_string())?;
    let recommendation = document.select(&rec_selector)
        .find(|el| el.text().collect::<String>().contains("Рекомендации"))
        .and_then(|el| el.next_siblings().next())
        .and_then(|el| {
            match el.value() {
                scraper::node::Node::Text(text) => Some(text.to_string()),
                _ => None,
            }
        })
        .map(|t| t.replace("Учитывая следующее:\n", "").trim().to_string());

    Ok(ServiceInfo {
        url: Some(url.to_string()),
        description,
        explained,
        recommendation,
        source: Some("scraped".to_string()),
        error: Some(false),
        message: None,
    })
}

async fn fetch_service_info_from_ai(service_name: &str) -> Result<ServiceInfo, String> {
    let api_key = std::env::var("GROK_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err("Grok API key not configured".to_string());
    }

    let prompt = format!("What is the Windows service \"{}\"?\n\nPlease provide:\n1. A brief description of what this service does\n2. A detailed explanation of its purpose and functionality\n3. A recommendation on whether users should disable it and why\n\nFormat your response as JSON with keys: \"description\", \"explained\", \"recommendation\"", service_name);

    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(10),
        client
            .post("https://api.x.ai/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": "grok-3",
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 1000,
                "temperature": 0.7,
                "stream": false
            }))
            .send()
    ).await.map_err(|_| "AI API request timed out".to_string())?
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("AI API error: {}", response.status()));
    }

    let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let ai_response = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid AI response format".to_string())?;

    // Try to parse as JSON
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(ai_response) {
        Ok(ServiceInfo {
            url: None,
            description: parsed["description"].as_str().map(|s| s.to_string()),
            explained: parsed["explained"].as_str().map(|s| s.to_string()),
            recommendation: parsed["recommendation"].as_str().map(|s| s.to_string()),
            source: Some("ai".to_string()),
            error: Some(false),
            message: None,
        })
    } else {
        // Fallback text parsing
        Ok(ServiceInfo {
            url: None,
            description: Some("AI-generated description".to_string()),
            explained: Some("AI-generated explanation".to_string()),
            recommendation: Some("AI-generated recommendation".to_string()),
            source: Some("ai".to_string()),
            error: Some(false),
            message: None,
        })
    }
}

fn save_services_info(services_info: &HashMap<String, ServiceInfo>) {
    if let Some(app_dir) = directories::ProjectDirs::from("com", "servicesmanager", "app") {
        let data_dir = app_dir.data_dir();
        if fs::create_dir_all(data_dir).is_ok() {
            let file_path = data_dir.join("services-info.json");
            if let Ok(json) = serde_json::to_string_pretty(services_info) {
                let _ = fs::write(file_path, json);
            }
        }
    }
}

fn load_services_info() -> HashMap<String, ServiceInfo> {
    if let Some(app_dir) = directories::ProjectDirs::from("com", "servicesmanager", "app") {
        let file_path = app_dir.data_dir().join("services-info.json");
        if let Ok(content) = fs::read_to_string(file_path) {
            if let Ok(data) = serde_json::from_str(&content) {
                return data;
            }
        }
    }
    HashMap::new()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
