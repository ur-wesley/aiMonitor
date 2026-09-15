use std::sync::Arc;

use parking_lot::Mutex;

use crate::auth::OAuthTokenRefresher;
use crate::providers::ProviderSet;
use crate::settings::load_settings;
use crate::usage::{UsageRefreshService, UsageStore};
use crate::yasb;

pub async fn run_export() -> Result<(), String> {
    let settings = load_settings();
    let store = Arc::new(UsageStore::new());
    let client = reqwest::Client::new();
    let oauth = OAuthTokenRefresher::new(client.clone());
    let providers = Arc::new(ProviderSet::new(client, oauth));
    let settings_arc = Arc::new(Mutex::new(settings));
    let refresh = UsageRefreshService::new(store.clone(), providers, settings_arc);

    let snapshot = refresh.refresh_once().await;
    let dto = yasb::to_dto(&snapshot);
    println!("{}", serde_json::to_string(&dto).map_err(|e| e.to_string())?);
    Ok(())
}
