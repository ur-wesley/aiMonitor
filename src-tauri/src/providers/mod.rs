mod antigravity;
mod cursor;
mod opencode_go;

pub use antigravity::AntigravityProvider;
pub use cursor::CursorProvider;
pub use opencode_go::OpenCodeGoProvider;

use crate::auth::OAuthTokenRefresher;
use crate::models::{AppSettings, ProviderUsage};

pub struct ProviderSet {
    pub cursor: CursorProvider,
    pub opencode_go: OpenCodeGoProvider,
    pub antigravity: AntigravityProvider,
}

impl ProviderSet {
    pub fn new(client: reqwest::Client, oauth: OAuthTokenRefresher) -> Self {
        Self {
            cursor: CursorProvider::new(client.clone()),
            opencode_go: OpenCodeGoProvider::new(client.clone()),
            antigravity: AntigravityProvider::new(client, oauth),
        }
    }

    pub async fn fetch_all(&self, settings: &AppSettings) -> Vec<ProviderUsage> {
        let (cursor, opencode, antigravity) = tokio::join!(
            fetch_safe(&self.cursor, settings),
            fetch_safe(&self.opencode_go, settings),
            fetch_safe(&self.antigravity, settings),
        );
        vec![cursor, opencode, antigravity]
    }
}

async fn fetch_safe<P>(provider: &P, settings: &AppSettings) -> ProviderUsage
where
    P: Provider + Send + Sync,
{
    match provider.fetch(settings).await {
        Ok(usage) => usage,
        Err(error) => ProviderUsage {
            provider_id: provider.provider_id().to_string(),
            display_name: provider.display_name().to_string(),
            windows: vec![],
            error: Some(error),
            fetched_at: chrono::Utc::now(),
        },
    }
}

pub trait Provider {
    fn provider_id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn fetch(&self, settings: &AppSettings) -> impl std::future::Future<Output = Result<ProviderUsage, String>> + Send;
}
