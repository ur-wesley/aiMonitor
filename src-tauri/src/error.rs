use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Serialize)]
pub struct StableError {
    pub code: String,
    pub message: String,
}

impl From<AppError> for StableError {
    fn from(value: AppError) -> Self {
        Self {
            code: "internal".into(),
            message: value.to_string(),
        }
    }
}

impl From<String> for StableError {
    fn from(message: String) -> Self {
        Self {
            code: "internal".into(),
            message,
        }
    }
}
