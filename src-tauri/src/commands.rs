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
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<String, StableError> {
    let mut current = state.settings.lock();
    current.refresh_interval_seconds = settings.refresh_interval_seconds.max(60);
    current.local_api_enabled = settings.local_api_enabled;
    current.local_api_port = settings.local_api_port;
    current.opencode_go_api_key = settings
        .opencode_go_api_key
        .filter(|s| !s.is_empty());
    current.cursor_state_db_path = settings
        .cursor_state_db_path
        .filter(|s| !s.is_empty());
    current.antigravity_state_db_path = settings
        .antigravity_state_db_path
        .filter(|s| !s.is_empty());
    current.low_usage_notifications_enabled = settings.low_usage_notifications_enabled;
    current.start_with_windows = settings.start_with_windows;

    save_settings(&current)?;

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

    let cursor = if read_sqlite_value(&cursor_state_db(&settings), "cursorAuth/accessToken").is_some() {
        "Token found"
    } else {
        "Not signed in"
    };

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
    let usage = app.get_webview_window("usage").ok_or_else(|| "usage window missing".to_string())?;

    if usage.is_visible().unwrap_or(false) {
        usage.hide().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let state = app.state::<AppState>();
    state.usage_popup.arm_show();
    state.usage_popup.place_at_cursor(&usage).map_err(|e| e.to_string())?;
    usage.show().map_err(|e| e.to_string())?;
    usage.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn place_usage_window(app: AppHandle) -> Result<(), StableError> {
    let usage = app.get_webview_window("usage").ok_or_else(|| "usage window missing".to_string())?;
    app.state::<AppState>()
        .usage_popup
        .place_from_anchor(&usage)
        .map_err(|e| e.to_string())?;
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
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
