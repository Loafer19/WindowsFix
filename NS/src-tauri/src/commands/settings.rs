use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::State;

use crate::models::{AppState, NotificationConfig, Settings};

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
    Ok(state.settings.lock().unwrap().clone())
}

#[tauri::command]
pub async fn set_settings(
    settings: Settings,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    crate::settings::set_autorun(settings.start_with_windows)?;
    let notif = state.notification_config.lock().unwrap().clone();
    crate::settings::save_settings(&settings, &notif)?;
    *state.settings.lock().unwrap() = settings;
    Ok(())
}

#[tauri::command]
pub async fn get_notification_config(
    state: State<'_, Arc<AppState>>,
) -> Result<NotificationConfig, String> {
    Ok(state.notification_config.lock().unwrap().clone())
}

#[tauri::command]
pub async fn set_notification_config(
    config: NotificationConfig,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let s = state.settings.lock().unwrap().clone();
    crate::settings::save_settings(&s, &config)?;
    *state.notification_config.lock().unwrap() = config;
    Ok(())
}

#[tauri::command]
pub async fn clear_all_data(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    *state.process_total_bytes.lock().unwrap() = std::collections::HashMap::new();
    *state.process_hourly.lock().unwrap() = std::collections::HashMap::new();
    *state.global_hourly.lock().unwrap() = VecDeque::new();
    state.current_hour_dl.store(0, Ordering::Relaxed);
    state.current_hour_ul.store(0, Ordering::Relaxed);

    super::persist_app_data(&state)
}
