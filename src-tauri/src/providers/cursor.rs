use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;

use crate::auth::read_sqlite_value;
use crate::models::{AppSettings, ProviderUsage, UsageWindowMetric};
use crate::paths::cursor_state_db;

use super::Provider;

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub struct CursorProvider {
    client: Client,
}

impl CursorProvider {
    pub const fn new(client: Client) -> Self {
        Self { client }
    }
}

impl Provider for CursorProvider {
    fn provider_id(&self) -> &'static str {
        "cursor"
    }

    fn display_name(&self) -> &'static str {
        "Cursor"
    }

    async fn fetch(&self, settings: &AppSettings) -> Result<ProviderUsage, String> {
        let db_path = cursor_state_db(settings);
        let token = read_sqlite_value(&db_path, "cursorAuth/accessToken")
            .ok_or_else(|| "Not signed in to Cursor".to_string())?;

        let user_id =
            extract_user_id(&token).ok_or_else(|| "Could not read Cursor session".to_string())?;

        let response = self
            .client
            .get("https://cursor.com/api/usage-summary")
            .header("User-Agent", USER_AGENT)
            .header(
                "Cookie",
                format!("WorkosCursorSessionToken={user_id}::{token}"),
            )
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err("Cursor session expired".into());
        }
        if !response.status().is_success() {
            return Err(format!("Cursor API error ({})", response.status()));
        }

        let body: CursorUsageSummaryResponse = response.json().await.map_err(|e| e.to_string())?;
        let windows = parse_usage_windows(&body)?;

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
#[serde(rename_all = "camelCase")]
struct CursorUsageSummaryResponse {
    #[serde(alias = "billing_cycle_end")]
    billing_cycle_end: Option<String>,
    #[serde(alias = "individual_usage")]
    individual_usage: Option<CursorIndividualUsage>,
    #[serde(alias = "auto_model_selected_display_message")]
    auto_model_selected_display_message: Option<String>,
    #[serde(alias = "named_model_selected_display_message")]
    named_model_selected_display_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CursorIndividualUsage {
    plan: Option<CursorPlanUsage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
struct CursorPlanUsage {
    #[serde(alias = "total_percent_used")]
    total_percent_used: Option<f64>,
    #[serde(alias = "auto_percent_used")]
    auto_percent_used: Option<f64>,
    #[serde(alias = "api_percent_used")]
    api_percent_used: Option<f64>,
}

fn parse_usage_windows(body: &CursorUsageSummaryResponse) -> Result<Vec<UsageWindowMetric>, String> {
    let plan = body.individual_usage.as_ref().and_then(|u| u.plan.as_ref());

    let mut total = plan.and_then(|p| p.total_percent_used);
    let mut auto = plan.and_then(|p| p.auto_percent_used);
    let mut api = plan.and_then(|p| p.api_percent_used);

    if total.is_none() {
        auto = parse_percent(body.auto_model_selected_display_message.as_ref());
        api = parse_percent(body.named_model_selected_display_message.as_ref());
        total = match (auto, api) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            _ => None,
        };
    }

    let reset = body
        .billing_cycle_end
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc));

    let mut windows = Vec::new();
    if let Some(v) = total {
        windows.push(UsageWindowMetric {
            label: "Total".into(),
            value: v,
            is_remaining_percent: false,
            resets_at: reset,
        });
    }
    if let Some(v) = auto {
        windows.push(UsageWindowMetric {
            label: "Auto".into(),
            value: v,
            is_remaining_percent: false,
            resets_at: reset,
        });
    }
    if let Some(v) = api {
        windows.push(UsageWindowMetric {
            label: "API".into(),
            value: v,
            is_remaining_percent: false,
            resets_at: reset,
        });
    }

    if windows.is_empty() {
        return Err("No usage data in Cursor response".into());
    }

    Ok(windows)
}

fn extract_user_id(jwt: &str) -> Option<String> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let payload = parts[1];
    let padded = match payload.len() % 4 {
        0 => payload.to_string(),
        n => format!("{}{}", payload, "=".repeat(4 - n)),
    };
    let decoded = URL_SAFE_NO_PAD
        .decode(padded.as_str())
        .or_else(|_| STANDARD.decode(padded.replace('-', "+").replace('_', "/")))
        .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    json.get("sub")?.as_str().map(std::string::ToString::to_string)
}

fn parse_percent(message: Option<&String>) -> Option<f64> {
    let message = message.filter(|s| !s.is_empty())?;
    let re = Regex::new(r"(\d+(?:\.\d+)?)\s*%").ok()?;
    re.captures(message)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAMEL_FIXTURE: &str = "{\"billingCycleEnd\":\"2026-04-01T00:00:00Z\",\"individualUsage\":{\"plan\":{\"totalPercentUsed\":42.5,\"autoPercentUsed\":30.0,\"apiPercentUsed\":12.5}}}";

    const SNAKE_FIXTURE: &str = "{\"billing_cycle_end\":\"2026-04-01T00:00:00Z\",\"individual_usage\":{\"plan\":{\"total_percent_used\":55.0,\"auto_percent_used\":40.0,\"api_percent_used\":15.0}}}";

    const MESSAGE_FIXTURE: &str = "{\"autoModelSelectedDisplayMessage\":\"You have used 25% of your auto quota\",\"namedModelSelectedDisplayMessage\":\"You have used 10% of your API quota\"}";

    #[test]
    fn parses_camel_case_usage_summary() {
        let body: CursorUsageSummaryResponse = serde_json::from_str(CAMEL_FIXTURE).unwrap();
        let windows = parse_usage_windows(&body).unwrap();
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0].label, "Total");
        assert!((windows[0].value - 42.5).abs() < f64::EPSILON);
        assert!(windows[0].resets_at.is_some());
    }

    #[test]
    fn parses_snake_case_usage_summary() {
        let body: CursorUsageSummaryResponse = serde_json::from_str(SNAKE_FIXTURE).unwrap();
        let windows = parse_usage_windows(&body).unwrap();
        assert_eq!(windows.len(), 3);
        assert!((windows[0].value - 55.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parses_display_message_fallback() {
        let body: CursorUsageSummaryResponse = serde_json::from_str(MESSAGE_FIXTURE).unwrap();
        let windows = parse_usage_windows(&body).unwrap();
        assert_eq!(windows.len(), 3);
        assert!((windows[0].value - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_response_errors() {
        let body: CursorUsageSummaryResponse = serde_json::from_str("{}").unwrap();
        assert!(parse_usage_windows(&body).is_err());
    }
}
