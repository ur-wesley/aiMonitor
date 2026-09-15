use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use parking_lot::Mutex;
use serde::Serialize;
use tokio::sync::oneshot;

use crate::models::{AppSettings, ProviderUsage, YasbExportDto};
use crate::usage::UsageStore;
use crate::yasb;

pub struct ApiServer {
    settings: Arc<Mutex<AppSettings>>,
    store: Arc<UsageStore>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
}

impl ApiServer {
    pub const fn new(settings: Arc<Mutex<AppSettings>>, store: Arc<UsageStore>) -> Self {
        Self {
            settings,
            store,
            shutdown: Mutex::new(None),
        }
    }

    pub fn restart(&self) {
        self.stop();
        self.start();
    }

    pub fn start(&self) {
        let settings = self.settings.lock().clone();
        if !settings.local_api_enabled {
            return;
        }

        let port = settings.local_api_port;
        let store = self.store.clone();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tauri::async_runtime::spawn(async move {
            let state = ApiState { store };
            let app = Router::new()
                .route("/health", get(health))
                .route("/api/v1/usage", get(full_usage))
                .route("/api/v1/usage/{provider_id}", get(provider_usage))
                .with_state(state);

            let Ok(listener) = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await else {
                return;
            };

            let server = axum::serve(listener, app).with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            });

            let _ = server.await;
        });

        *self.shutdown.lock() = Some(shutdown_tx);
    }

    pub fn stop(&self) {
        if let Some(tx) = self.shutdown.lock().take() {
            let _ = tx.send(());
        }
    }
}

#[derive(Clone)]
struct ApiState {
    store: Arc<UsageStore>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
    })
}

async fn full_usage(State(state): State<ApiState>) -> Json<YasbExportDto> {
    let snapshot = state.store.current();
    Json(yasb::to_dto(&snapshot))
}

async fn provider_usage(
    State(state): State<ApiState>,
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderUsage>, StatusCode> {
    let snapshot = state.store.current();
    let provider = snapshot
        .providers
        .into_iter()
        .find(|p| p.provider_id.eq_ignore_ascii_case(&provider_id));

    match provider {
        Some(p) => Ok(Json(p)),
        None => Err(StatusCode::NOT_FOUND),
    }
}
