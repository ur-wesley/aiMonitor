use std::fs;

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

use crate::models::AppSettings;
use crate::paths::settings_file_path;

use super::dpapi;

const fn default_true() -> bool {
    true
}

fn default_label_position() -> String {
    "beside".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredSettingsDto {
    refresh_interval_seconds: u32,
    local_api_enabled: bool,
    local_api_port: u16,
    #[serde(rename = "encryptedOpenCodeGoApiKey")]
    encrypted_opencode_go_api_key: Option<String>,
    cursor_state_db_path: Option<String>,
    antigravity_state_db_path: Option<String>,
    start_with_windows: bool,
    low_usage_notifications_enabled: bool,
    #[serde(default)]
    overlay_enabled: bool,
    #[serde(default = "default_true")]
    overlay_always_on_top: bool,
    #[serde(default = "default_true")]
    overlay_show_percentage: bool,
    #[serde(default)]
    overlay_items: Vec<String>,
    #[serde(default = "default_true")]
    overlay_show_labels: bool,
    #[serde(default = "default_label_position")]
    overlay_label_position: String,
    #[serde(default)]
    overlay_position_x: Option<i32>,
    #[serde(default)]
    overlay_position_y: Option<i32>,
}

pub fn load_settings() -> AppSettings {
    let path = settings_file_path();
    if !path.exists() {
        return AppSettings::default();
    }

    fs::read_to_string(&path).map_or_else(
        |_| AppSettings::default(),
        |json| parse_stored(&json).unwrap_or_default(),
    )
}

fn parse_stored(json: &str) -> Result<AppSettings, String> {
    let stored: StoredSettingsDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
    Ok(AppSettings {
        refresh_interval_seconds: stored.refresh_interval_seconds,
        local_api_enabled: stored.local_api_enabled,
        local_api_port: stored.local_api_port,
        opencode_go_api_key: decrypt_key(stored.encrypted_opencode_go_api_key.as_ref()),
        cursor_state_db_path: stored.cursor_state_db_path,
        antigravity_state_db_path: stored.antigravity_state_db_path,
        start_with_windows: stored.start_with_windows,
        low_usage_notifications_enabled: stored.low_usage_notifications_enabled,
        overlay_enabled: stored.overlay_enabled,
        overlay_always_on_top: stored.overlay_always_on_top,
        overlay_show_percentage: stored.overlay_show_percentage,
        overlay_items: stored.overlay_items,
        overlay_show_labels: stored.overlay_show_labels,
        overlay_label_position: stored.overlay_label_position,
        overlay_position_x: stored.overlay_position_x,
        overlay_position_y: stored.overlay_position_y,
    })
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let stored = StoredSettingsDto {
        refresh_interval_seconds: settings.refresh_interval_seconds,
        local_api_enabled: settings.local_api_enabled,
        local_api_port: settings.local_api_port,
        encrypted_opencode_go_api_key: encrypt_key(settings.opencode_go_api_key.as_deref()),
        cursor_state_db_path: settings.cursor_state_db_path.clone(),
        antigravity_state_db_path: settings.antigravity_state_db_path.clone(),
        start_with_windows: settings.start_with_windows,
        low_usage_notifications_enabled: settings.low_usage_notifications_enabled,
        overlay_enabled: settings.overlay_enabled,
        overlay_always_on_top: settings.overlay_always_on_top,
        overlay_show_percentage: settings.overlay_show_percentage,
        overlay_items: settings.overlay_items.clone(),
        overlay_show_labels: settings.overlay_show_labels,
        overlay_label_position: settings.overlay_label_position.clone(),
        overlay_position_x: settings.overlay_position_x,
        overlay_position_y: settings.overlay_position_y,
    };

    let json = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn encrypt_key(plain: Option<&str>) -> Option<String> {
    let plain = plain.filter(|s| !s.is_empty())?;
    dpapi::protect(plain.as_bytes())
        .ok()
        .map(|bytes| STANDARD.encode(bytes))
}

fn decrypt_key(encrypted: Option<&String>) -> Option<String> {
    let encrypted = encrypted.filter(|s| !s.is_empty())?;
    let bytes = STANDARD.decode(encrypted).ok()?;
    let plain = dpapi::unprotect(&bytes).ok()?;
    String::from_utf8(plain).ok()
}
