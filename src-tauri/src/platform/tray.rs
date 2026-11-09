use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager,
};

use crate::commands;
use crate::state::AppState;

pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let widget_item = MenuItem::with_id(app, "widget", "Toggle Mini Widget", true, None::<&str>)?;
    let refresh_item = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &widget_item,
            &refresh_item,
            &settings_item,
            &PredefinedMenuItem::separator(app)?,
            &quit_item,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(
            app.default_window_icon()
                .expect("default window icon missing")
                .clone(),
        )
        .tooltip("AI Usage")
        .show_menu_on_left_click(false)
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "widget" => {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = commands::toggle_overlay_window(app);
                    });
                }
                "refresh" => {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app.state::<AppState>();
                        let snapshot = state.refresh.refresh_once().await;
                        state.notifier.evaluate(&snapshot);
                    });
                }
                "settings" => {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = commands::show_settings_window(app);
                    });
                }
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    let _ = commands::toggle_usage_window(app);
                });
            }
        })
        .build(app)?;

    Ok(())
}
