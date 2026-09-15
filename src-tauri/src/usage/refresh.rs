use std::sync::Arc;

use chrono::Utc;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex as AsyncMutex;

use crate::models::{AppSettings, UsageSnapshot};
use crate::providers::ProviderSet;

use super::store::UsageStore;

pub struct UsageRefreshService {
    store: Arc<UsageStore>,
    providers: Arc<ProviderSet>,
    settings: Arc<Mutex<AppSettings>>,
    refresh_lock: AsyncMutex<()>,
    app: Mutex<Option<AppHandle>>,
}

impl UsageRefreshService {
    pub fn new(
        store: Arc<UsageStore>,
        providers: Arc<ProviderSet>,
        settings: Arc<Mutex<AppSettings>>,
    ) -> Self {
        Self {
            store,
            providers,
            settings,
            refresh_lock: AsyncMutex::new(()),
            app: Mutex::new(None),
        }
    }

    pub fn set_app(&self, app: AppHandle) {
        *self.app.lock() = Some(app);
    }

    pub async fn refresh_once(&self) -> UsageSnapshot {
        let _guard = self.refresh_lock.lock().await;
        let settings = self.settings.lock().clone();
        let providers = self.providers.fetch_all(&settings).await;
        let snapshot = UsageSnapshot {
            providers,
            fetched_at: Utc::now(),
        };
        self.store.update(snapshot.clone());

        if let Some(app) = self.app.lock().clone() {
            let _ = app.emit("usage-updated", &snapshot);
        }

        snapshot
    }

    pub fn spawn_poll_loop(self: Arc<Self>) {
        tauri::async_runtime::spawn(async move {
            let _ = self.refresh_once().await;

            loop {
                let interval = {
                    let settings = self.settings.lock();
                    settings.refresh_interval_seconds.max(60)
                };

                tokio::time::sleep(tokio::time::Duration::from_secs(u64::from(interval))).await;
                let _ = self.refresh_once().await;
            }
        });
    }
}
