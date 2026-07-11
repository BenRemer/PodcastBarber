use axum::{extract::State, Json};
use axum::http::StatusCode;
use serde_json::Value;
use crate::{state::AppState, error::AppError};
use crate::extractors::AudioUpload;

pub async fn handle_upload(
    State(state): State<AppState>,
    upload: AudioUpload,
) -> Result<(StatusCode, Json<Value>), AppError> {
    tracing::info!("Shipping '{}' ({} bytes) to Whisper sidecar", upload.file_name, upload.data.len());
    
    let json_result = state.whisper_service
        .transcribe_audio(upload.file_name, upload.content_type, upload.data)
        .await
        .map_err(|e| {
            tracing::error!("Whisper inference failed: {}", e);
            AppError::InternalServerError("Inference engine failure".into())
        })?;

    Ok((StatusCode::OK, Json(json_result)))
}