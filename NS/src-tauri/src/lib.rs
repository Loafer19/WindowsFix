mod capture;
mod commands;
mod db;
mod history;
mod metrics;
mod models;
mod settings;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use tauri::Manager;

use commands::*;
use metrics::Metrics;
use models::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize structured logging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    tracing::info!("NetSentry starting up");

    // Load persisted settings and 24h global history
    let (saved_settings, saved_notif) = settings::load_settings_and_notifications();
    let saved_data = db::load();

    // Restore global hourly history, padding with zeros for any hours the app was offline
    let current_hour = history::current_unix_hour();
    let mut global_hourly: VecDeque<(u64, u64)> = saved_data.global_hourly.into_iter().collect();
    if saved_data.saved_at_hour > 0 {
        let offline_hours = (current_hour.saturating_sub(saved_data.saved_at_hour) as usize)
            .min(history::HOURLY_BUCKETS);
        for _ in 0..offline_hours {
            global_hourly.push_back((0, 0));
            if global_hourly.len() > history::HOURLY_BUCKETS {
                global_hourly.pop_front();
            }
        }
    }

    // Restore per-exe hourly history, padding missed hours with zeros
    let mut process_hourly: HashMap<String, VecDeque<(u64, u64)>> = saved_data
        .process_hourly
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect();
    if saved_data.saved_at_hour > 0 {
        let offline_hours = (current_hour.saturating_sub(saved_data.saved_at_hour) as usize)
            .min(history::HOURLY_BUCKETS);
        for deque in process_hourly.values_mut() {
            for _ in 0..offline_hours {
                deque.push_back((0, 0));
                if deque.len() > history::HOURLY_BUCKETS {
                    deque.pop_front();
                }
            }
        }
    }

    // Restore blocked exe paths
    let blocked_exes: HashSet<String> = saved_data.blocked_exes.into_iter().collect();

    let app_state = Arc::new(AppState::new(
        saved_settings,
        saved_notif,
        global_hourly,
        saved_data.process_totals,
        blocked_exes,
        process_hourly,
    ));

    let metrics = Arc::new(Metrics::new());

    tauri::Builder::default()
        .manage(app_state)
        .manage(metrics)
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};

            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("NetSentry")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Hide main window on startup if "Start minimized" is configured
            let state = app.state::<Arc<AppState>>();
            if state.settings.lock().unwrap().start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_capture,
            stop_capture,
            get_network_stats,
            get_processes,
            set_global_limit,
            set_process_limit,
            block_process,
            unblock_process,
            kill_process,
            get_process_history,
            get_24h_totals,
            get_metrics,
            get_settings,
            set_settings,
            get_notification_config,
            set_notification_config,
            clear_all_data,
            show_native_notification,
            check_windivert_status,
            install_windivert,
            start_windivert_service,
            exit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
