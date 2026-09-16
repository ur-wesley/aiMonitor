use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWindowMetric {
    pub label: String,
    pub value: f64,
    pub is_remaining_percent: bool,
    pub resets_at: Option<DateTime<Utc>>,
}

impl UsageWindowMetric {
    pub fn remaining_percent(&self) -> f64 {
        if self.is_remaining_percent {
            self.value
        } else {
            100.0 - self.value
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsage {
    pub provider_id: String,
    pub display_name: String,
    pub windows: Vec<UsageWindowMetric>,
    pub error: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub providers: Vec<ProviderUsage>,
    pub fetched_at: DateTime<Utc>,
}

impl UsageSnapshot {
    pub fn empty() -> Self {
        Self {
            providers: vec![],
            fetched_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub refresh_interval_seconds: u32,
    pub local_api_enabled: bool,
    pub local_api_port: u16,
    pub opencode_go_api_key: Option<String>,
    pub cursor_state_db_path: Option<String>,
    pub antigravity_state_db_path: Option<String>,
    pub start_with_windows: bool,
    pub low_usage_notifications_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 300,
            local_api_enabled: true,
            local_api_port: 6736,
            opencode_go_api_key: None,
            cursor_state_db_path: None,
            antigravity_state_db_path: None,
            start_with_windows: false,
            low_usage_notifications_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialHealth {
    pub cursor: String,
    pub opencode_go: String,
    pub antigravity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YasbExportDto {
    pub cursor: YasbCursorDto,
    #[serde(rename = "opencode_go")]
    pub opencode_go: YasbOpenCodeGoDto,
    pub antigravity: YasbAntigravityDto,
    pub max_used: f64,
    pub status: String,
    pub fetched_at: String,
    pub label: String,
    pub label_alt: String,
    pub tooltip: String,
    pub status_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YasbCursorDto {
    pub total: Option<f64>,
    pub auto: Option<f64>,
    pub api: Option<f64>,
    pub resets_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YasbOpenCodeGoDto {
    pub rolling: Option<f64>,
    pub weekly: Option<f64>,
    pub monthly: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YasbAntigravityDto {
    pub gemini_5h: Option<f64>,
    pub gemini_weekly: Option<f64>,
    pub claude_5h: Option<f64>,
    pub claude_weekly: Option<f64>,
    pub error: Option<String>,
}
