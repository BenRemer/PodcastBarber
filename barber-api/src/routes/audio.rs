use crate::extractors::AudioUpload;
use crate::services::transcribe::types::TranscribeJob;
use crate::{error::AppError, state::AppState};
use axum::extract::State;
use axum::http::StatusCode;
use uuid::Uuid;

pub async fn handle_episode_transcribe(
    State(state): State<AppState>,
    upload: AudioUpload,
) -> Result<StatusCode, AppError> {
    tracing::info!(
        "Shipping '{}' ({} bytes) to Whisper sidecar",
        upload.file_name,
        upload.data.len()
    );

    let job = TranscribeJob {
        tracking_id: Uuid::new_v4(),
        file_name: upload.file_name,
        content_type: upload.content_type,
        data: upload.data,
    };

    state.whisper_service.transcribe_audio(job).await?;

    Ok(StatusCode::ACCEPTED)
}
