use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::{state::AppState, error::AppError};
use crate::routes::rss::models::EpisodesResponse;

#[derive(Deserialize)]
pub struct FeedRequest {
    pub feed_url: String,
    pub guid: Option<String>,
    pub size: Option<usize>,
}

pub async fn get_latest_mp3(
    State(state): State<AppState>,
    Json(payload): Json<FeedRequest>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Downloading latest mp3 of {}", payload.feed_url);

    let saved_path = state.rssfeed_service
        .download_latest_episode(&payload.feed_url)
        .await?;


    Ok(Json(json!({
        "status": "success",
        "file_path": saved_path
    })))
}

pub async fn list_episodes(
    state: State<AppState>,
    Json(payload): Json<FeedRequest>,
) -> Result<Json<EpisodesResponse>, AppError> {
    let size = payload.size.unwrap_or(20);
    tracing::info!("Listing {} episodes of {}",size, payload.feed_url);

    let list = state.rssfeed_service
        .list_episodes(&payload.feed_url, size)
        .await?;

    Ok(Json(EpisodesResponse {
        total: list.len(),
        items: list,
    }))
}

pub async fn save_episode(
    state: State<AppState>,
    Json(payload): Json<FeedRequest>,
) -> Result<Json<Value>, AppError> {
    let guid = payload.guid.ok_or_else(|| AppError::InvalidInput("GUID missing from payload".into()))?;

    tracing::info!("Downloading episode of {} with id {}", payload.feed_url, guid);

    let episode_path = state.rssfeed_service
        .download_episode(&payload.feed_url, &guid)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "file_path": episode_path
    })))
}