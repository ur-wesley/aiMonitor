use std::sync::Arc;

use parking_lot::Mutex;

use crate::api::ApiServer;
use crate::auth::OAuthTokenRefresher;
use crate::models::AppSettings;
use crate::providers::ProviderSet;
use crate::platform::usage_popup::UsagePopupGuard;
use crate::usage::{LowUsageNotifier, UsageRefreshService, UsageStore};

pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub store: Arc<UsageStore>,
    pub refresh: Arc<UsageRefreshService>,
    pub api: Arc<ApiServer>,
    pub notifier: Arc<LowUsageNotifier>,
    pub usage_popup: UsagePopupGuard,
}

impl AppState {
    pub fn new() -> Self {
        let mut loaded = crate::settings::load_settings();
        let registry_enabled = crate::platform::autostart::is_enabled();
        if loaded.start_with_windows || registry_enabled {
            let _ = crate::platform::autostart::set_enabled(true);
        }
        loaded.start_with_windows = crate::platform::autostart::is_enabled();
        let settings = Arc::new(Mutex::new(loaded));
        let store = Arc::new(UsageStore::new());
        let client = reqwest::Client::new();
        let oauth = OAuthTokenRefresher::new(client.clone());
        let providers = Arc::new(ProviderSet::new(client, oauth));
        let refresh = Arc::new(UsageRefreshService::new(
            store.clone(),
            providers,
            settings.clone(),
        ));
        let api = Arc::new(ApiServer::new(settings.clone(), store.clone()));
        let notifier = Arc::new(LowUsageNotifier::new(settings.clone()));

        Self {
            settings,
            store,
            refresh,
            api,
            notifier,
            usage_popup: UsagePopupGuard::new(),
        }
    }
}
