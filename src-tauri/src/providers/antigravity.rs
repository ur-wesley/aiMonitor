use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::auth::OAuthTokenRefresher;
use crate::models::{AppSettings, ProviderUsage, UsageWindowMetric};

use super::Provider;

const USER_AGENT: &str = "antigravity";
const BASE_URLS: [&str; 3] = [
    "https://daily-cloudcode-pa.googleapis.com",
    "https://daily-cloudcode-pa.sandbox.googleapis.com",
    "https://cloudcode-pa.googleapis.com",
];

pub struct AntigravityProvider {
    client: Client,
    oauth: OAuthTokenRefresher,
}

impl AntigravityProvider {
    pub const fn new(client: Client, oauth: OAuthTokenRefresher) -> Self {
        Self { client, oauth }
    }
}

impl Provider for AntigravityProvider {
    fn provider_id(&self) -> &'static str {
        "antigravity"
    }

    fn display_name(&self) -> &'static str {
        "Antigravity"
    }

    async fn fetch(&self, settings: &AppSettings) -> Result<ProviderUsage, String> {
        let token = self
            .oauth
            .resolve_access_token(settings)
            .await
            .ok_or_else(|| "Not signed in to Antigravity".to_string())?;

        let project_id = self.load_project_id(&token).await;
        let body = self.load_quota(&token, project_id.as_deref()).await?;

        let mut windows = Vec::new();
        for group in body.all_groups() {
            let is_gemini = group.name().to_lowercase().contains("gemini");
            let prefix = if is_gemini { "Gemini" } else { "Claude/GPT" };
            let buckets = group.all_buckets();

            if buckets.len() >= 2 {
                let (five_hour, weekly) = order_quota_buckets(&buckets, is_gemini);
                add_remaining(&mut windows, format!("{prefix} 5h"), five_hour);
                add_remaining(&mut windows, format!("{prefix} Weekly"), weekly);
            } else if buckets.len() == 1 {
                add_remaining(&mut windows, format!("{prefix} 5h"), buckets[0]);
            }
        }

        if windows.is_empty() {
            return Err("No quota buckets in Antigravity response".into());
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

impl AntigravityProvider {
    async fn load_project_id(&self, token: &str) -> Option<String> {
        let payload = json!({
            "metadata": { "ideType": "ANTIGRAVITY" }
        });

        for base_url in BASE_URLS {
            let response = self
                .client
                .post(format!("{base_url}/v1internal:loadCodeAssist"))
                .bearer_auth(token)
                .header("User-Agent", USER_AGENT)
                .json(&payload)
                .send()
                .await;

            let Ok(response) = response else {
                continue;
            };

            if !response.status().is_success() {
                continue;
            }

            let body: LoadCodeAssistResponse = match response.json().await {
                Ok(b) => b,
                Err(_) => continue,
            };
            if let Some(project) = body.cloudaicompanion_project.filter(|p| !p.is_empty()) {
                return Some(project);
            }
        }
        None
    }

    async fn load_quota(&self, token: &str, project_id: Option<&str>) -> Result<AntigravityQuotaResponse, String> {
        let project_body = match project_id {
            Some(id) => json!({ "project": id }),
            None => json!({}),
        };

        for base_url in BASE_URLS {
            let response = self
                .client
                .post(format!("{base_url}/v1internal:retrieveUserQuotaSummary"))
                .bearer_auth(token)
                .header("User-Agent", USER_AGENT)
                .json(&project_body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !response.status().is_success() {
                continue;
            }

            let body: AntigravityQuotaResponse = response.json().await.map_err(|e| e.to_string())?;
            if !body.all_groups().is_empty() {
                return Ok(body);
            }
        }

        Err("Could not fetch Antigravity quota".into())
    }
}

#[derive(Debug, Deserialize)]
struct LoadCodeAssistResponse {
    cloudaicompanion_project: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AntigravityQuotaResponse {
    groups: Option<Vec<AntigravityQuotaGroup>>,
    #[serde(rename = "quotaGroups")]
    quota_groups: Option<Vec<AntigravityQuotaGroup>>,
    #[serde(rename = "quota_groups")]
    quota_groups_snake: Option<Vec<AntigravityQuotaGroup>>,
}

impl AntigravityQuotaResponse {
    fn all_groups(&self) -> Vec<&AntigravityQuotaGroup> {
        self.groups
            .as_deref()
            .or(self.quota_groups.as_deref())
            .or(self.quota_groups_snake.as_deref())
            .map(|g| g.iter().collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
struct AntigravityQuotaGroup {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "display_name")]
    display_name_snake: Option<String>,
    buckets: Option<Vec<AntigravityQuotaBucket>>,
    #[serde(rename = "quotaBuckets")]
    quota_buckets: Option<Vec<AntigravityQuotaBucket>>,
    #[serde(rename = "quota_buckets")]
    quota_buckets_snake: Option<Vec<AntigravityQuotaBucket>>,
}

impl AntigravityQuotaGroup {
    fn name(&self) -> String {
        self.display_name
            .clone()
            .or(self.display_name_snake.clone())
            .unwrap_or_else(|| "Unknown".into())
    }

    fn all_buckets(&self) -> Vec<&AntigravityQuotaBucket> {
        self.buckets
            .as_deref()
            .or(self.quota_buckets.as_deref())
            .or(self.quota_buckets_snake.as_deref())
            .map(|b| b.iter().collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
struct AntigravityQuotaBucket {
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "remaining_fraction")]
    remaining_fraction_snake: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
    #[serde(rename = "reset_time")]
    reset_time_snake: Option<String>,
}

impl AntigravityQuotaBucket {
    fn fraction(&self) -> Option<f64> {
        self.remaining_fraction.or(self.remaining_fraction_snake)
    }

    fn reset(&self) -> Option<String> {
        self.reset_time.clone().or(self.reset_time_snake.clone())
    }
}

fn order_quota_buckets<'a>(
    buckets: &'a [&'a AntigravityQuotaBucket],
    is_gemini: bool,
) -> (&'a AntigravityQuotaBucket, &'a AntigravityQuotaBucket) {
    let with_reset: Vec<&AntigravityQuotaBucket> = buckets
        .iter()
        .filter_map(|b| parse_reset(b).map(|_| *b))
        .collect();

    if with_reset.len() >= 2 {
        let sorted = {
            let mut copy = with_reset.clone();
            copy.sort_by_key(|b| parse_reset(b).unwrap_or_default());
            copy
        };
        return (sorted[0], sorted[1]);
    }

    if is_gemini {
        (buckets[1], buckets[0])
    } else {
        (buckets[0], buckets[1])
    }
}

fn parse_reset(bucket: &AntigravityQuotaBucket) -> Option<DateTime<Utc>> {
    bucket
        .reset()
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
}

fn add_remaining(windows: &mut Vec<UsageWindowMetric>, label: String, bucket: &AntigravityQuotaBucket) {
    if let Some(fraction) = bucket.fraction() {
        let remaining = (fraction * 100.0).round();
        windows.push(UsageWindowMetric {
            label,
            value: remaining,
            is_remaining_percent: true,
            resets_at: parse_reset(bucket),
        });
    }
}
