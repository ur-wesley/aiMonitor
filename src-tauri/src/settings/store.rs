use std::fs;

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

use crate::models::AppSettings;
use crate::paths::settings_file_path;

use super::dpapi;

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
}

pub fn load_settings() -> AppSettings {
    let path = settings_file_path();
    if !path.exists() {
        return AppSettings::default();
    }

    match fs::read_to_string(&path) {
        Ok(json) => parse_stored(&json).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
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
    };

    let json = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn encrypt_key(plain: Option<&str>) -> Option<String> {
    let plain = plain.filter(|s| !s.is_empty())?;
    match dpapi::protect(plain.as_bytes()) {
        Ok(bytes) => Some(STANDARD.encode(bytes)),
        Err(_) => None,
    }
}

fn decrypt_key(encrypted: Option<&String>) -> Option<String> {
    let encrypted = encrypted.filter(|s| !s.is_empty())?;
    let bytes = STANDARD.decode(encrypted).ok()?;
    let plain = dpapi::unprotect(&bytes).ok()?;
    String::from_utf8(plain).ok()
}
