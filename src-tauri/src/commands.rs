#![allow(clippy::needless_pass_by_value)]

use tauri::{AppHandle, Manager, State};

use crate::auth::read_sqlite_value;
use crate::error::StableError;
use crate::models::{AppSettings, CredentialHealth, UsageSnapshot};
use crate::paths::{antigravity_state_db_candidates, cursor_state_db, opencode_auth_paths};
use crate::platform::autostart;
use crate::platform::window_style;
use crate::settings::save_settings;
use crate::state::AppState;
use crate::yasb;

#[tauri::command]
pub fn get_usage_snapshot(state: State<'_, AppState>) -> UsageSnapshot {
    state.store.current()
}

#[tauri::command]
pub async fn refresh_usage(state: State<'_, AppState>) -> Result<UsageSnapshot, StableError> {
    let snapshot = state.refresh.refresh_once().await;
    state.notifier.evaluate(&snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.lock().clone()
}

#[tauri::command]
pub fn save_app_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<String, StableError> {
    let mut current = state.settings.lock();
    current.refresh_interval_seconds = settings.refresh_interval_seconds.max(60);
    current.local_api_enabled = settings.local_api_enabled;
    current.local_api_port = settings.local_api_port;
    current.opencode_go_api_key = settings.opencode_go_api_key.filter(|s| !s.is_empty());
    current.cursor_state_db_path = settings.cursor_state_db_path.filter(|s| !s.is_empty());
    current.antigravity_state_db_path =
        settings.antigravity_state_db_path.filter(|s| !s.is_empty());
    current.low_usage_notifications_enabled = settings.low_usage_notifications_enabled;
    current.start_with_windows = settings.start_with_windows;
    current.overlay_enabled = settings.overlay_enabled;
    current.overlay_always_on_top = settings.overlay_always_on_top;
    current.overlay_show_percentage = settings.overlay_show_percentage;
    current.overlay_items = settings.overlay_items;
    current.overlay_show_labels = settings.overlay_show_labels;
    current.overlay_label_position = settings.overlay_label_position;
    current.overlay_position_x = settings.overlay_position_x;
    current.overlay_position_y = settings.overlay_position_y;

    save_settings(&current)?;

    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.set_always_on_top(settings.overlay_always_on_top);
        if settings.overlay_enabled {
            let _ = overlay.show();
        } else {
            let _ = overlay.hide();
        }
    }

    if let Err(error) = autostart::set_enabled(settings.start_with_windows) {
        drop(current);
        state.api.restart();
        return Ok(format!(
            "Settings saved, but autostart could not be updated: {error}"
        ));
    }

    drop(current);
    state.api.restart();
    Ok("Saved.".into())
}

#[tauri::command]
pub fn check_credentials(state: State<'_, AppState>) -> CredentialHealth {
    let settings = state.settings.lock().clone();

    let cursor = read_sqlite_value(&cursor_state_db(&settings), "cursorAuth/accessToken").map_or(
        "Not signed in",
        |raw| {
            if crate::providers::resolve_cursor_token(&raw, &settings).is_some() {
                "Token found"
            } else {
                "Token found (decryption failed)"
            }
        },
    );

    let opencode = if settings.opencode_go_api_key.is_some() || read_api_key_from_files() {
        "Key found"
    } else {
        "No key found"
    };

    let antigravity = if crate::auth::read_antigravity_token().is_some()
        || antigravity_state_db_candidates(&settings)
            .iter()
            .any(|p| read_sqlite_value(p, "antigravityAuthStatus").is_some())
    {
        "Credentials found"
    } else {
        "Not signed in"
    };

    CredentialHealth {
        cursor: cursor.into(),
        opencode_go: opencode.into(),
        antigravity: antigravity.into(),
    }
}

fn read_api_key_from_files() -> bool {
    for path in opencode_auth_paths() {
        if path.exists() {
            return true;
        }
    }
    false
}

#[tauri::command]
pub fn get_tray_tooltip(state: State<'_, AppState>) -> String {
    let dto = yasb::to_dto(&state.store.current());
    if dto.max_used >= 95.0 {
        format!("AI Usage — critical ({:.0}% used)", dto.max_used)
    } else if dto.max_used >= 80.0 {
        format!("AI Usage — warning ({:.0}% used)", dto.max_used)
    } else {
        "AI Usage".into()
    }
}

#[tauri::command]
pub fn toggle_usage_window(app: AppHandle) -> Result<(), StableError> {
    let usage = app
        .get_webview_window("usage")
        .ok_or_else(|| "usage window missing".to_string())?;

    if usage.is_visible().unwrap_or(false) {
        usage.hide().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let state = app.state::<AppState>();
    state.usage_popup.arm_show();
    state.usage_popup.place_at_cursor(&usage)?;
    usage.show().map_err(|e| e.to_string())?;
    usage.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn place_usage_window(app: AppHandle) -> Result<(), StableError> {
    let usage = app
        .get_webview_window("usage")
        .ok_or_else(|| "usage window missing".to_string())?;
    app.state::<AppState>()
        .usage_popup
        .place_from_anchor(&usage)?;
    Ok(())
}

#[tauri::command]
pub fn show_settings_window(app: AppHandle) -> Result<(), StableError> {
    let settings = app
        .get_webview_window("settings")
        .ok_or_else(|| "settings window missing".to_string())?;
    window_style::apply_glass(&settings);
    settings.show().map_err(|e| e.to_string())?;
    settings.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn hide_usage_window(app: AppHandle) -> Result<(), StableError> {
    if let Some(usage) = app.get_webview_window("usage") {
        usage.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_overlay_window(app: AppHandle) -> Result<(), StableError> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or_else(|| "overlay window missing".to_string())?;

    let state = app.state::<AppState>();
    let is_visible = overlay.is_visible().unwrap_or(false);

    if is_visible {
        overlay.hide().map_err(|e| e.to_string())?;
        let mut settings = state.settings.lock();
        settings.overlay_enabled = false;
        let res = save_settings(&settings);
        drop(settings);
        let _ = res;
    } else {
        let settings = state.settings.lock().clone();
        let _ = overlay.set_always_on_top(settings.overlay_always_on_top);
        if let (Some(x), Some(y)) = (settings.overlay_position_x, settings.overlay_position_y) {
            let _ = overlay.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x, y)));
        }
        overlay.show().map_err(|e| e.to_string())?;
        drop(settings);
        let mut settings = state.settings.lock();
        settings.overlay_enabled = true;
        let res = save_settings(&settings);
        drop(settings);
        let _ = res;
    }
    Ok(())
}

#[tauri::command]
pub fn show_overlay_window(app: AppHandle) -> Result<(), StableError> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or_else(|| "overlay window missing".to_string())?;
    let state = app.state::<AppState>();
    let settings = state.settings.lock().clone();
    let _ = overlay.set_always_on_top(settings.overlay_always_on_top);
    if let (Some(x), Some(y)) = (settings.overlay_position_x, settings.overlay_position_y) {
        let _ = overlay.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x, y)));
    }
    overlay.show().map_err(|e| e.to_string())?;
    drop(settings);
    let mut settings = state.settings.lock();
    settings.overlay_enabled = true;
    let res = save_settings(&settings);
    drop(settings);
    let _ = res;
    Ok(())
}

#[tauri::command]
pub fn hide_overlay_window(app: AppHandle) -> Result<(), StableError> {
    if let Some(overlay) = app.get_webview_window("overlay") {
        overlay.hide().map_err(|e| e.to_string())?;
        let state = app.state::<AppState>();
        let mut settings = state.settings.lock();
        settings.overlay_enabled = false;
        let res = save_settings(&settings);
        drop(settings);
        let _ = res;
    }
    Ok(())
}

#[tauri::command]
pub fn save_overlay_position(app: AppHandle, x: i32, y: i32) -> Result<(), StableError> {
    let state = app.state::<AppState>();
    let mut settings = state.settings.lock();
    settings.overlay_position_x = Some(x);
    settings.overlay_position_y = Some(y);
    let res = save_settings(&settings);
    drop(settings);
    res?;
    Ok(())
}

#[tauri::command]
#[allow(clippy::cast_possible_truncation)]
pub fn reset_overlay_position(app: AppHandle) -> Result<(), StableError> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or_else(|| "overlay window missing".to_string())?;

    if let Ok(Some(monitor)) = overlay.current_monitor() {
        let screen_size = monitor.size();
        let scale_factor = monitor.scale_factor();
        let x = 224.0f64.mul_add(-scale_factor, f64::from(screen_size.width)).max(0.0) as i32;
        let y = (24.0 * scale_factor) as i32;
        let _ = overlay.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x, y)));

        let state = app.state::<AppState>();
        let mut settings = state.settings.lock();
        settings.overlay_position_x = Some(x);
        settings.overlay_position_y = Some(y);
        let res = save_settings(&settings);
        drop(settings);
        res?;
    }
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
