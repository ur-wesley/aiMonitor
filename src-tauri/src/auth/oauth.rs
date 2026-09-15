use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::models::AppSettings;
use crate::paths::{antigravity_oauth_token_file, antigravity_state_db_candidates, gemini_oauth_creds_file};

use super::cred::read_antigravity_token;
use super::sqlite::read_sqlite_value;

const CLIENT_ID: &str = "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "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";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

pub struct OAuthTokenRefresher {
    client: Client,
}

impl OAuthTokenRefresher {
    pub const fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn resolve_access_token(&self, settings: &AppSettings) -> Option<String> {
        if let Some(credential) = read_antigravity_token() {
            if let Some(token) = self.resolve_from_json(&credential).await {
                return Some(token);
            }
        }

        for path in [gemini_oauth_creds_file(), antigravity_oauth_token_file()] {
            if !path.exists() {
                continue;
            }
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Some(token) = self.resolve_from_json(&contents).await {
                    return Some(token);
                }
            }
        }

        Self::read_vscdb_api_key(settings)
    }

    async fn resolve_from_json(&self, token_json: &str) -> Option<String> {
        let stored = parse_stored_token(token_json)?;
        if !stored.access_token.is_empty() && !is_expired(&stored) {
            return Some(stored.access_token);
        }
        if stored.refresh_token.is_empty() {
            return is_expired(&stored).then_some(stored.access_token).filter(|t| !t.is_empty());
        }
        self.refresh(&stored.refresh_token).await
    }

    fn read_vscdb_api_key(settings: &AppSettings) -> Option<String> {
        for db_path in antigravity_state_db_candidates(settings) {
            let json = read_sqlite_value(&db_path, "antigravityAuthStatus")?;
            let status: AntigravityAuthStatus = serde_json::from_str(&json).ok()?;
            if let Some(key) = status.api_key.filter(|k| !k.is_empty()) {
                return Some(key);
            }
        }
        None
    }

    async fn refresh(&self, refresh_token: &str) -> Option<String> {
        let response = self
            .client
            .post(TOKEN_URL)
            .form(&[
                ("client_id", CLIENT_ID),
                ("client_secret", CLIENT_SECRET),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            return None;
        }

        let body: OAuthTokenResponse = response.json().await.ok()?;
        body.access_token
    }
}

#[derive(Debug, Deserialize)]
struct AntigravityAuthStatus {
    #[serde(rename = "apiKey")]
    api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: Option<String>,
}

#[derive(Debug)]
struct StoredOAuthToken {
    access_token: String,
    refresh_token: String,
    expiry: Option<String>,
    expires_at: Option<i64>,
    expiry_date: Option<i64>,
}

fn parse_stored_token(token_json: &str) -> Option<StoredOAuthToken> {
    if token_json.starts_with("ya29.") {
        return Some(StoredOAuthToken {
            access_token: token_json.to_string(),
            refresh_token: String::new(),
            expiry: None,
            expires_at: None,
            expiry_date: None,
        });
    }

    let raw: RawStoredToken = serde_json::from_str(token_json).ok()?;
    let nested = raw.token.as_deref();
    Some(StoredOAuthToken {
        access_token: nested
            .and_then(|t| t.access_token.clone())
            .or(raw.access_token)
            .unwrap_or_default(),
        refresh_token: nested
            .and_then(|t| t.refresh_token.clone())
            .or(raw.refresh_token)
            .unwrap_or_default(),
        expiry: nested.and_then(|t| t.expiry.clone()).or(raw.expiry),
        expires_at: nested.and_then(|t| t.expires_at).or(raw.expires_at),
        expiry_date: nested.and_then(|t| t.expiry_date).or(raw.expiry_date),
    })
}

#[derive(Debug, Deserialize)]
struct RawStoredToken {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expiry: Option<String>,
    expires_at: Option<i64>,
    expiry_date: Option<i64>,
    token: Option<Box<Self>>,
}

fn is_expired(token: &StoredOAuthToken) -> bool {
    let threshold = Utc::now() + chrono::Duration::minutes(2);

    if let Some(expiry_ms) = token.expiry_date.filter(|v| *v > 0) {
        return DateTime::<Utc>::from_timestamp_millis(expiry_ms)
            .is_some_and(|e| e <= threshold);
    }

    if let Some(unix) = token.expires_at.filter(|v| *v > 0) {
        return DateTime::<Utc>::from_timestamp(unix, 0)
            .is_some_and(|e| e <= threshold);
    }

    if let Some(expiry) = &token.expiry {
        return DateTime::parse_from_rfc3339(expiry)
            .is_ok_and(|e| e <= threshold);
    }

    false
}
