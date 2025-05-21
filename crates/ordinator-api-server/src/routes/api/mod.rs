pub mod v1;

use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError
{
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError
{
    fn into_response(self) -> axum::response::Response
    {
        match self {
            AppError::Anyhow(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error" : error.to_string()})),
            )
                .into_response(),
        }
    }
}
