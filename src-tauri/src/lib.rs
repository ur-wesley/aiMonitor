mod api;
mod auth;
mod commands;
mod error;
pub mod export;
mod models;
mod paths;
mod platform;
mod providers;
mod settings;
mod state;
mod usage;
mod yasb;

use tauri::Manager;
use crate::platform::crash;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    crash::register_panic_hook();

    let state = AppState::new();
    let refresh = state.refresh.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_usage_snapshot,
            commands::refresh_usage,
            commands::get_settings,
            commands::save_app_settings,
            commands::check_credentials,
            commands::get_tray_tooltip,
            commands::toggle_usage_window,
            commands::place_usage_window,
            commands::show_settings_window,
            commands::hide_usage_window,
            commands::quit_app,
        ])
        .setup(move |app| {
            let state = app.state::<AppState>();
            crate::platform::toast::WindowsToast::initialize();
            state.api.start();
            state.refresh.clone().spawn_poll_loop();
            refresh.set_app(app.handle().clone());
            platform::tray::setup_tray(app)?;

            platform::window_style::apply_glass_for_label(app.handle(), "usage");
            platform::window_style::apply_glass_for_label(app.handle(), "settings");

            if let Some(main) = app.get_webview_window("main") {
                let _ = main.hide();
            }

            let refresh = app.state::<AppState>().refresh.clone();
            let notifier = app.state::<AppState>().notifier.clone();
            tauri::async_runtime::spawn(async move {
                let snapshot = refresh.refresh_once().await;
                notifier.evaluate(&snapshot);
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            match window.label() {
                "usage" => {
                    if matches!(event, tauri::WindowEvent::Focused(false)) {
                        let state = window.state::<AppState>();
                        if state.usage_popup.may_hide() {
                            let _ = window.hide();
                        }
                    }
                    if matches!(event, tauri::WindowEvent::Resized(_)) {
                        let app = window.app_handle();
                        let state = app.state::<AppState>();
                        if let Some(usage) = app.get_webview_window("usage") {
                            let _ = state.usage_popup.place_from_anchor(&usage);
                        }
                    }
                }
                "settings" => {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
