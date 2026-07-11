use crate::error::AppError;
use axum::Json;
use axum::http::StatusCode;
use serde_json::{Value, json};

pub async fn handle_ping() -> Result<(StatusCode, Json<Value>), AppError> {
    tracing::info!("Received ping request");

    Ok((StatusCode::OK, Json(json!({ "payload": "PONG" }))))
}
