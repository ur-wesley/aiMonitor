use std::path::PathBuf;

use crate::models::AppSettings;

pub fn settings_file_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub fn app_data_dir() -> PathBuf {
    dirs_path("APPDATA").join("aiMonitor")
}

pub fn crash_log_path() -> PathBuf {
    dirs_path("LOCALAPPDATA").join("aiMonitor").join("crash.log")
}

fn dirs_path(env: &str) -> PathBuf {
    std::env::var(env).map_or_else(|_| PathBuf::from("."), PathBuf::from)
}

pub fn cursor_state_db(settings: &AppSettings) -> PathBuf {
    if let Some(path) = &settings.cursor_state_db_path {
        return expand_env(path);
    }
    dirs_path("APPDATA")
        .join("Cursor")
        .join("User")
        .join("globalStorage")
        .join("state.vscdb")
}

pub fn opencode_auth_paths() -> Vec<PathBuf> {
    let profile = dirs_path("USERPROFILE");
    let local = dirs_path("LOCALAPPDATA");
    vec![
        profile.join(".local").join("share").join("opencode").join("auth.json"),
        local.join("opencode").join("auth.json"),
    ]
}

pub fn gemini_oauth_creds_file() -> PathBuf {
    dirs_path("USERPROFILE")
        .join(".gemini")
        .join("oauth_creds.json")
}

pub fn antigravity_oauth_token_file() -> PathBuf {
    dirs_path("USERPROFILE")
        .join(".gemini")
        .join("antigravity-cli")
        .join("antigravity-oauth-token")
}

pub fn antigravity_state_db_candidates(settings: &AppSettings) -> Vec<PathBuf> {
    if let Some(path) = &settings.antigravity_state_db_path {
        return vec![expand_env(path)];
    }

    let app_data = dirs_path("APPDATA");
    let folders = ["Antigravity IDE", "Antigravity", "antigravity"];
    let mut paths = Vec::new();
    for folder in folders {
        let path = app_data
            .join(folder)
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        if path.exists() {
            paths.push(path);
        }
    }
    if paths.is_empty() {
        for folder in ["Antigravity IDE", "Antigravity"] {
            paths.push(
                app_data
                    .join(folder)
                    .join("User")
                    .join("globalStorage")
                    .join("state.vscdb"),
            );
        }
    }
    paths
}

fn expand_env(path: &str) -> PathBuf {
    let mut result = path.to_string();
    for (key, value) in std::env::vars() {
        result = result.replace(&format!("%{key}%"), &value);
    }
    PathBuf::from(result)
}
