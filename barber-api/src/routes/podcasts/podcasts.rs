use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::{state::AppState, error::AppError};

#[derive(Deserialize)]
pub struct FeedRequest { // todo rename
    pub feed_url: String,
    pub guid: Option<String>,
    pub size: Option<usize>,
}

// pub async fn get_latest_mp3(
//     State(state): State<AppState>,
//     Json(payload): Json<FeedRequest>,
// ) -> Result<Json<Value>, AppError> {
//     tracing::info!("Downloading latest mp3 of {}", payload.feed_url);
//
//     let saved_path = state.rssfeed_service
//         .download_latest_episode(&payload.feed_url)
//         .await?;
//
//
//     Ok(Json(json!({
//         "status": "success",
//         "file_path": saved_path
//     })))
// }

pub async fn save_episode(
    state: State<AppState>,
    Json(payload): Json<FeedRequest>,
) -> Result<Json<Value>, AppError> {
    let guid = payload.guid.ok_or_else(|| AppError::BadRequest("GUID missing from payload".into()))?;

    tracing::info!("Downloading episode of {} with id {}", payload.feed_url, guid);

    let episode_path = state.podcast_service
        .download_episode(&payload.feed_url, &guid)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "file_path": episode_path
    })))
}

pub async fn subscribe_to_podcast(
    state: State<AppState>,
    Json(payload): Json<FeedRequest>,
) -> Result<Json<Value>, AppError> {

    let _podcast = state.podcast_service
        .subscribe_podcast(&payload.feed_url)
        .await
        .map_err(|e| {
            tracing::error!("Failed to subscribe to podcast: {:?}", e);
            AppError::InternalServerError("Failure to subscribe".into())
        })?;

    Ok(Json(json!({"status": "success"})))
}