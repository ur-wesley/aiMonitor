use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::models::{AppSettings, ProviderUsage, UsageWindowMetric};
use crate::paths::opencode_auth_paths;

use super::Provider;

pub struct OpenCodeGoProvider {
    client: Client,
}

impl OpenCodeGoProvider {
    pub const fn new(client: Client) -> Self {
        Self { client }
    }
}

impl Provider for OpenCodeGoProvider {
    fn provider_id(&self) -> &'static str {
        "opencode_go"
    }

    fn display_name(&self) -> &'static str {
        "OpenCode Go"
    }

    async fn fetch(&self, settings: &AppSettings) -> Result<ProviderUsage, String> {
        let api_key = settings
            .opencode_go_api_key
            .clone()
            .or_else(read_api_key)
            .ok_or_else(|| "No OpenCode Go API key found".to_string())?;

        let response = self
            .client
            .get("https://opencode.ai/zen/go/v1/usage")
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status() == reqwest::StatusCode::FORBIDDEN {
            return Err("No Go subscription on this key".into());
        }
        if !response.status().is_success() {
            return Err(format!("OpenCode API error ({})", response.status()));
        }

        let body: OpenCodeGoUsageResponse = response.json().await.map_err(|e| e.to_string())?;
        let usage = body.usage.ok_or_else(|| "Empty OpenCode response".to_string())?;

        let mut windows = Vec::new();
        add_window(&mut windows, "5h", usage.rolling.as_ref());
        add_window(&mut windows, "Weekly", usage.weekly.as_ref());
        add_window(&mut windows, "Monthly", usage.monthly.as_ref());

        if windows.is_empty() {
            return Err("No quota windows in OpenCode response".into());
        }

        Ok(ProviderUsage {
            provider_id: self.provider_id().into(),
            display_name: self.display_name().into(),
            windows,
            error: None,
            fetched_at: Utc::now(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct OpenCodeGoUsageResponse {
    usage: Option<OpenCodeGoUsageBuckets>,
}

#[derive(Debug, Deserialize)]
struct OpenCodeGoUsageBuckets {
    rolling: Option<OpenCodeGoUsageWindow>,
    weekly: Option<OpenCodeGoUsageWindow>,
    monthly: Option<OpenCodeGoUsageWindow>,
}

#[derive(Debug, Deserialize)]
struct OpenCodeGoUsageWindow {
    percent: Option<f64>,
    resets_at: Option<String>,
}

fn add_window(
    windows: &mut Vec<UsageWindowMetric>,
    label: &str,
    bucket: Option<&OpenCodeGoUsageWindow>,
) {
    if let Some(bucket) = bucket {
        if let Some(percent) = bucket.percent {
            let reset = bucket
                .resets_at
                .as_deref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&Utc));
            windows.push(UsageWindowMetric {
                label: label.to_string(),
                value: percent,
                is_remaining_percent: false,
                resets_at: reset,
            });
        }
    }
}

fn read_api_key() -> Option<String> {
    for path in opencode_auth_paths() {
        if !path.exists() {
            continue;
        }
        let contents = std::fs::read_to_string(&path).ok()?;
        let node: Value = serde_json::from_str(&contents).ok()?;
        for key in ["opencode-go", "opencode_go", "OpenCode Go", "go"] {
            if let Some(api_key) = extract_api_key(&node[key]) {
                return Some(api_key);
            }
        }
        if let Some(api_key) = find_sk_key(&node) {
            return Some(api_key);
        }
    }
    None
}

fn extract_api_key(node: &Value) -> Option<String> {
    match node {
        Value::String(text) if !text.is_empty() => Some(text.clone()),
        Value::Object(obj) => {
            for key in ["key", "apiKey", "api_key"] {
                if let Some(Value::String(text)) = obj.get(key) {
                    if !text.is_empty() {
                        return Some(text.clone());
                    }
                }
            }
            for (_, value) in obj {
                if let Some(found) = extract_api_key(value) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

fn find_sk_key(node: &Value) -> Option<String> {
    match node {
        Value::String(text) if text.starts_with("sk-") => Some(text.clone()),
        Value::Object(obj) => {
            for (_, value) in obj {
                if let Some(found) = find_sk_key(value) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}
