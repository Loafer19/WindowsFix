mod ai;
mod config;
mod database;
mod history;
mod models;
mod startup_apps;
mod windows_api;
mod windows_services;

use std::sync::Mutex;
use std::time::SystemTime;

use tauri::State;

use ai::fetch_service_info_from_ai;
use config::AppConfig;
use history::HistoryEntry;
use models::{AppState, ServiceInfo, ServicesCache, StartupApp, StartupLocation, WindowsService};
use windows_services::{
    disable_windows_service, get_default_service_info, get_windows_services,
    restart_windows_service, set_windows_service_startup_type, start_windows_service,
    stop_windows_service,
};

/// Maximum number of history entries loaded from the database on startup and returned per query.
const MAX_HISTORY_ENTRIES: u32 = 500;

#[tauri::command]
async fn get_services(state: State<'_, AppState>) -> Result<Vec<WindowsService>, String> {
    let needs_refresh = {
        let cache = state.services_cache.lock().unwrap();
        let ttl = cache.ttl;
        cache.data.is_empty()
            || SystemTime::now()
                .duration_since(cache.last_updated)
                .unwrap_or(std::time::Duration::MAX)
                > ttl
    };

    if needs_refresh {
        match refresh_services_cache(&state).await {
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
    match refresh_services_cache(&state).await {
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
    let db = state.db.lock().unwrap();
    database::save_service_info(&db, &service_name, &info);

    Ok(info)
}

#[tauri::command]
async fn disable_service(
    service_name: String,
    state: State<'_, AppState>,
) -> Result<WindowsService, String> {
    let old_startup = get_cached_startup_type(&state, &service_name);
    let service_name_clone = service_name.clone();
    match tokio::task::spawn_blocking(move || disable_windows_service(&service_name_clone)).await {
        Ok(result) => match result {
            Ok(updated_service) => {
                update_cache_service(&state, &service_name, &updated_service);
                record_service_history(
                    &state,
                    &service_name,
                    "disable",
                    old_startup.as_deref(),
                    Some(&updated_service.startup_type),
                );
                Ok(updated_service)
            }
            Err(e) => Err(format!("Failed to disable service: {}", e)),
        },
        Err(e) => Err(format!("Task panicked: {:?}", e)),
    }
}

#[tauri::command]
async fn start_service(
    service_name: String,
    state: State<'_, AppState>,
) -> Result<WindowsService, String> {
    let old_status = get_cached_status(&state, &service_name);
    let service_name_clone = service_name.clone();
    match tokio::task::spawn_blocking(move || start_windows_service(&service_name_clone)).await {
        Ok(result) => match result {
            Ok(updated_service) => {
                update_cache_service(&state, &service_name, &updated_service);
                record_service_history(
                    &state,
                    &service_name,
                    "start",
                    old_status.as_deref(),
                    Some(&updated_service.status),
                );
                Ok(updated_service)
            }
            Err(e) => Err(format!("Failed to start service: {}", e)),
        },
        Err(e) => Err(format!("Task panicked: {:?}", e)),
    }
}

#[tauri::command]
async fn stop_service(
    service_name: String,
    state: State<'_, AppState>,
) -> Result<WindowsService, String> {
    let old_status = get_cached_status(&state, &service_name);
    let service_name_clone = service_name.clone();
    match tokio::task::spawn_blocking(move || stop_windows_service(&service_name_clone)).await {
        Ok(result) => match result {
            Ok(updated_service) => {
                update_cache_service(&state, &service_name, &updated_service);
                record_service_history(
                    &state,
                    &service_name,
                    "stop",
                    old_status.as_deref(),
                    Some(&updated_service.status),
                );
                Ok(updated_service)
            }
            Err(e) => Err(format!("Failed to stop service: {}", e)),
        },
        Err(e) => Err(format!("Task panicked: {:?}", e)),
    }
}

#[tauri::command]
async fn restart_service(
    service_name: String,
    state: State<'_, AppState>,
) -> Result<WindowsService, String> {
    let old_status = get_cached_status(&state, &service_name);
    let service_name_clone = service_name.clone();
    match tokio::task::spawn_blocking(move || restart_windows_service(&service_name_clone)).await {
        Ok(result) => match result {
            Ok(updated_service) => {
                update_cache_service(&state, &service_name, &updated_service);
                record_service_history(
                    &state,
                    &service_name,
                    "restart",
                    old_status.as_deref(),
                    Some(&updated_service.status),
                );
                Ok(updated_service)
            }
            Err(e) => Err(format!("Failed to restart service: {}", e)),
        },
        Err(e) => Err(format!("Task panicked: {:?}", e)),
    }
}

#[tauri::command]
async fn set_startup_type(
    service_name: String,
    startup_type: String,
    state: State<'_, AppState>,
) -> Result<WindowsService, String> {
    let old_startup = get_cached_startup_type(&state, &service_name);
    let sn = service_name.clone();
    let st = startup_type.clone();
    match tokio::task::spawn_blocking(move || set_windows_service_startup_type(&sn, &st)).await {
        Ok(result) => match result {
            Ok(updated_service) => {
                update_cache_service(&state, &service_name, &updated_service);
                record_service_history(
                    &state,
                    &service_name,
                    "set_startup_type",
                    old_startup.as_deref(),
                    Some(&updated_service.startup_type),
                );
                Ok(updated_service)
            }
            Err(e) => Err(format!("Failed to set startup type: {}", e)),
        },
        Err(e) => Err(format!("Task panicked: {:?}", e)),
    }
}

#[tauri::command]
async fn get_startup_apps(state: State<'_, AppState>) -> Result<Vec<StartupApp>, String> {
    match tokio::task::spawn_blocking(startup_apps::list_startup_apps).await {
        Ok(result) => match result {
            Ok(apps) => {
                *state.startup_apps_cache.lock().unwrap() = apps.clone();
                Ok(apps)
            }
            Err(e) => Err(e),
        },
        Err(e) => Err(format!("Task panicked: {:?}", e)),
    }
}

#[tauri::command]
async fn remove_startup_app(
    app: StartupApp,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let name = app.name.clone();
    let location = app.location.clone();
    let app_clone = app.clone();
    match tokio::task::spawn_blocking(move || {
        startup_apps::remove_startup_app(&app_clone.name, &app_clone.location)
    })
    .await
    {
        Ok(result) => match result {
            Ok(()) => {
                let entry = HistoryEntry::startup_app(&name, "remove", location.as_str());
                state.history.lock().unwrap().push(entry.clone());
                database::append_history(&state.db.lock().unwrap(), &entry);
                Ok(())
            }
            Err(e) => Err(e),
        },
        Err(e) => Err(format!("Task panicked: {:?}", e)),
    }
}

#[tauri::command]
async fn add_startup_app(
    app: StartupApp,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app_clone = app.clone();
    match tokio::task::spawn_blocking(move || {
        startup_apps::add_startup_app(&app_clone)
    })
    .await
    {
        Ok(result) => match result {
            Ok(()) => {
                let entry = HistoryEntry::startup_app(&app.name, "add", app.location.as_str());
                state.history.lock().unwrap().push(entry.clone());
                database::append_history(&state.db.lock().unwrap(), &entry);
                Ok(())
            }
            Err(e) => Err(e),
        },
        Err(e) => Err(format!("Task panicked: {:?}", e)),
    }
}

/// Return history entries, optionally filtered by type ("service" | "startupApp").
#[tauri::command]
async fn get_history(
    filter: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<HistoryEntry>, String> {
    let history = state.history.lock().unwrap();
    let entries: Vec<HistoryEntry> = match filter.as_deref() {
        Some("service") => history
            .iter()
            .filter(|e| matches!(e, HistoryEntry::Service(_)))
            .cloned()
            .collect(),
        Some("startupApp") => history
            .iter()
            .filter(|e| matches!(e, HistoryEntry::StartupApp(_)))
            .cloned()
            .collect(),
        _ => history.clone(),
    };
    Ok(entries)
}

#[tauri::command]
async fn get_history_by_type(
    entry_type: String,
    state: State<'_, AppState>,
) -> Result<Vec<HistoryEntry>, String> {
    get_history(Some(entry_type), state).await
}

#[tauri::command]
async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let mut history = state.history.lock().unwrap();
    history.clear();
    let db = state.db.lock().unwrap();
    database::clear_history(&db);
    Ok(())
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn update_cache_service(state: &AppState, service_name: &str, updated: &WindowsService) {
    let mut cache = state.services_cache.lock().unwrap();
    if let Some(service) = cache.data.iter_mut().find(|s| s.name == service_name) {
        service.status = updated.status.clone();
        service.startup_type = updated.startup_type.clone();
    }
}

fn get_cached_status(state: &AppState, service_name: &str) -> Option<String> {
    let cache = state.services_cache.lock().unwrap();
    cache.data.iter().find(|s| s.name == service_name).map(|s| s.status.clone())
}

fn get_cached_startup_type(state: &AppState, service_name: &str) -> Option<String> {
    let cache = state.services_cache.lock().unwrap();
    cache.data.iter().find(|s| s.name == service_name).map(|s| s.startup_type.clone())
}

fn record_service_history(
    state: &AppState,
    service_name: &str,
    action: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
) {
    let entry = HistoryEntry::service(service_name, action, old_value, new_value);
    state.history.lock().unwrap().push(entry.clone());
    database::append_history(&state.db.lock().unwrap(), &entry);
}

async fn refresh_services_cache(state: &AppState) -> Result<Vec<WindowsService>, String> {
    match get_windows_services().await {
        Ok(services) => {
            let mut info_map = state.services_info.lock().unwrap();
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

            let db = state.db.lock().unwrap();
            database::save_all_service_info(&db, &info_map);

            Ok(processed_services)
        }
        Err(e) => Err(format!("Failed to get Windows services: {}", e)),
    }
}

// ─── Entry Point ─────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = dotenvy::dotenv() {
        println!("Warning: Could not load .env file: {}", e);
    }

    let config = AppConfig::from_env();

    let db = match database::open_db() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Warning: Could not open database, falling back to in-memory: {}", e);
            database::open_memory_db().expect("In-memory SQLite should always work")
        }
    };

    let services_info = database::load_service_info(&db, config.service_info_ttl.as_secs());

    let history = database::load_history(&db, MAX_HISTORY_ENTRIES);

    let app_state = AppState {
        services_cache: Mutex::new(ServicesCache {
            data: Vec::new(),
            last_updated: SystemTime::UNIX_EPOCH,
            ttl: config.cache_ttl,
        }),
        services_info: Mutex::new(services_info),
        history: Mutex::new(history),
        startup_apps_cache: Mutex::new(Vec::new()),
        db: Mutex::new(db),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_services,
            refresh_services,
            reload_service_info,
            disable_service,
            start_service,
            stop_service,
            restart_service,
            set_startup_type,
            get_startup_apps,
            remove_startup_app,
            add_startup_app,
            get_history,
            get_history_by_type,
            clear_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
