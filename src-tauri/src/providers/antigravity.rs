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
            let group_name = group.name();
            let is_gemini = group_name.to_lowercase().contains("gemini");
            let prefix = if is_gemini {
                "Gemini"
            } else if group_name.to_lowercase().contains("claude")
                || group_name.to_lowercase().contains("gpt")
            {
                "Claude/GPT"
            } else {
                group_name.as_str()
            };
            let buckets = group.all_buckets();

            let mut resolved_any = false;
            for bucket in &buckets {
                let window_tag = if bucket.window.as_deref() == Some("5h")
                    || bucket
                        .bucket_id
                        .as_deref()
                        .is_some_and(|id| id.contains("5h"))
                {
                    Some("5h")
                } else if bucket.window.as_deref() == Some("weekly")
                    || bucket
                        .bucket_id
                        .as_deref()
                        .is_some_and(|id| id.contains("weekly"))
                {
                    Some("Weekly")
                } else if bucket.window.as_deref() == Some("daily")
                    || bucket
                        .bucket_id
                        .as_deref()
                        .is_some_and(|id| id.contains("daily"))
                {
                    Some("Daily")
                } else {
                    bucket.window.as_deref().filter(|s| !s.is_empty())
                };
                if let Some(tag) = window_tag {
                    add_remaining(&mut windows, format!("{prefix} {tag}"), bucket);
                    resolved_any = true;
                }
            }

            if !resolved_any {
                if buckets.len() >= 2 {
                    let (five_hour, weekly) = order_quota_buckets(&buckets, is_gemini);
                    add_remaining(&mut windows, format!("{prefix} 5h"), five_hour);
                    add_remaining(&mut windows, format!("{prefix} Weekly"), weekly);
                } else if buckets.len() == 1 {
                    add_remaining(&mut windows, format!("{prefix} 5h"), buckets[0]);
                }
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

    async fn load_quota(
        &self,
        token: &str,
        project_id: Option<&str>,
    ) -> Result<AntigravityQuotaResponse, String> {
        let project_body = project_id.map_or_else(|| json!({}), |id| json!({ "project": id }));

        for base_url in BASE_URLS {
            let response = self
                .client
                .post(format!("{base_url}/v1internal:retrieveUserQuotaSummary"))
                .bearer_auth(token)
                .header("User-Agent", USER_AGENT)
                .json(&project_body)
                .send()
                .await;

            let Ok(response) = response else {
                continue;
            };

            if !response.status().is_success() {
                continue;
            }

            let Ok(body): Result<AntigravityQuotaResponse, _> = response.json().await else {
                continue;
            };
            if !body.all_groups().is_empty() {
                return Ok(body);
            }
        }

        Err("Could not fetch Antigravity quota".into())
    }
}

#[derive(Debug, Deserialize)]
struct LoadCodeAssistResponse {
    #[serde(alias = "cloudaicompanionProject")]
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
            .or_else(|| self.display_name_snake.clone())
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
    #[serde(rename = "bucketId", alias = "bucket_id")]
    bucket_id: Option<String>,
    window: Option<String>,
}

impl AntigravityQuotaBucket {
    fn fraction(&self) -> Option<f64> {
        self.remaining_fraction.or(self.remaining_fraction_snake)
    }

    fn reset(&self) -> Option<String> {
        self.reset_time
            .clone()
            .or_else(|| self.reset_time_snake.clone())
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

fn add_remaining(
    windows: &mut Vec<UsageWindowMetric>,
    label: String,
    bucket: &AntigravityQuotaBucket,
) {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parses_load_code_assist_camel_and_snake() {
        let camel = "{\"cloudaicompanionProject\":\"test-project-1\"}";
        let res: LoadCodeAssistResponse = serde_json::from_str(camel).unwrap();
        assert_eq!(
            res.cloudaicompanion_project.as_deref(),
            Some("test-project-1")
        );

        let snake = "{\"cloudaicompanion_project\":\"test-project-2\"}";
        let res2: LoadCodeAssistResponse = serde_json::from_str(snake).unwrap();
        assert_eq!(
            res2.cloudaicompanion_project.as_deref(),
            Some("test-project-2")
        );
    }

    #[test]
    fn parses_quota_summary_with_window_and_bucket_id() {
        let json_data = r#"{
            "groups": [
                {
                    "displayName": "Claude and GPT models",
                    "buckets": [
                        {
                            "bucketId": "3p-weekly",
                            "window": "weekly",
                            "remainingFraction": 0.85
                        },
                        {
                            "bucketId": "3p-5h",
                            "window": "5h",
                            "remainingFraction": 1.0
                        }
                    ]
                }
            ]
        }"#;
        let quota: AntigravityQuotaResponse = serde_json::from_str(json_data).unwrap();
        let groups = quota.all_groups();
        assert_eq!(groups.len(), 1);
        let buckets = groups[0].all_buckets();
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].bucket_id.as_deref(), Some("3p-weekly"));
        assert_eq!(buckets[0].window.as_deref(), Some("weekly"));
    }
}
