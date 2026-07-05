use axum::Json;
use serde_json::{json, Value};
use crate::error::AppError;

pub async fn handle_ping() -> Result<Json<Value>, AppError> {
    tracing::info!("Received ping request");

    Ok(Json(json!({ "status": "PONG" })))
}