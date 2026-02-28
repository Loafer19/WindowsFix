mod ai;
mod models;
mod windows_services;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use tauri::State;

use ai::fetch_service_info_from_ai;
use models::{AppState, ServiceInfo, ServicesCache, WindowsService};
use windows_services::{
    disable_windows_service, get_default_service_info, get_windows_services, load_services_info,
    save_services_info,
};

#[tauri::command]
async fn get_services(state: State<'_, AppState>) -> Result<Vec<WindowsService>, String> {
    let needs_refresh = {
        let cache = state.services_cache.lock().unwrap();
        cache.data.is_empty()
            || SystemTime::now()
                .duration_since(cache.last_updated)
                .unwrap_or(Duration::MAX)
                > cache.ttl
    };

    if needs_refresh {
        match refresh_services_cache(&state.services_info).await {
            Ok(new_data) => {
                let mut cache = state.services_cache.lock().unwrap();
                cache.data = new_data;
                cache.last_updated = SystemTime::now();
            }
            Err(e) => return Err(format!("Failed to refresh services cache: {}", e)),
        }
    }

    let cache = state.services_cache.lock().unwrap();
    Ok(cache.data.clone())
}

#[tauri::command]
async fn refresh_services(state: State<'_, AppState>) -> Result<(), String> {
    match refresh_services_cache(&state.services_info).await {
        Ok(new_data) => {
            let mut cache = state.services_cache.lock().unwrap();
            cache.data = new_data;
            cache.last_updated = SystemTime::now();
            Ok(())
        }
        Err(e) => Err(format!("Failed to refresh services cache: {}", e)),
    }
}

#[tauri::command]
async fn reload_service_info(
    service_name: String,
    state: State<'_, AppState>,
) -> Result<ServiceInfo, String> {
    let info = match fetch_service_info_from_ai(&service_name).await {
        Ok(ai_info) => ai_info,
        Err(_) => {
            let services_info = state.services_info.lock().unwrap();
            if let Some(existing) = services_info.get(&service_name) {
                if existing.explained.is_some() || existing.recommendation.is_some() {
                    return Ok(existing.clone());
                }
            }
            get_default_service_info(&service_name, &services_info)
        }
    };

    let mut cache = state.services_cache.lock().unwrap();
    if let Some(service) = cache.data.iter_mut().find(|s| s.name == service_name) {
        service.info = info.clone();
    }
    drop(cache);

    let mut services_info = state.services_info.lock().unwrap();
    services_info.insert(service_name.clone(), info.clone());

    Ok(info)
}

#[tauri::command]
async fn disable_service(
    service_name: String,
    state: State<'_, AppState>,
) -> Result<WindowsService, String> {
    let service_name_clone = service_name.clone();
    match tokio::task::spawn_blocking(move || disable_windows_service(&service_name_clone)).await {
        Ok(result) => match result {
            Ok(updated_service) => {
                let mut cache = state.services_cache.lock().unwrap();
                if let Some(service) = cache.data.iter_mut().find(|s| s.name == service_name) {
                    service.status = updated_service.status.clone();
                    service.startup_type = updated_service.startup_type.clone();
                }
                Ok(updated_service)
            }
            Err(e) => Err(format!("Failed to disable service: {}", e)),
        },
        Err(e) => Err(format!("Task panicked: {:?}", e)),
    }
}

async fn refresh_services_cache(
    services_info: &Mutex<HashMap<String, ServiceInfo>>,
) -> Result<Vec<WindowsService>, String> {
    match get_windows_services().await {
        Ok(services) => {
            let mut info_map = services_info.lock().unwrap();
            let processed_services: Vec<WindowsService> = services
                .into_iter()
                .map(|service| {
                    let name = service.name.split('_').next().unwrap_or(&service.name).to_string();

                    let info = if let Some(existing_info) = info_map.get(&name) {
                        if existing_info.explained.is_some() || existing_info.recommendation.is_some() {
                            existing_info.clone()
                        } else {
                            get_default_service_info(&name, &info_map)
                        }
                    } else {
                        let default_info = get_default_service_info(&name, &info_map);
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

            save_services_info(&info_map);

            Ok(processed_services)
        }
        Err(e) => Err(format!("Failed to get Windows services: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = dotenvy::dotenv() {
        println!("Warning: Could not load .env file: {}", e);
    }

    let services_info = load_services_info();

    let app_state = AppState {
        services_cache: Mutex::new(ServicesCache {
            data: Vec::new(),
            last_updated: SystemTime::UNIX_EPOCH,
            ttl: Duration::from_secs(300),
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
