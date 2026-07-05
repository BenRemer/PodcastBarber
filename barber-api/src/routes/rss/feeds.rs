use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::{state::AppState, error::AppError};
use crate::routes::rss::models::ItemsResponse;

#[derive(Deserialize)]
pub struct FeedRequest {
    pub feed_url: String,
    pub guid: Option<String>,
}

pub async fn get_latest_mp3(
    State(state): State<AppState>,
    Json(payload): Json<FeedRequest>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Downloading latest mp3 of {}", payload.feed_url);

    let saved_path = state.rssfeed_service
        .download_latest_episode(&payload.feed_url, "/app/downloads")
        .await?;


    Ok(Json(json!({
        "status": "success",
        "file_path": saved_path
    })))
}

pub async fn list_episodes(
    state: State<AppState>,
    Json(payload): Json<FeedRequest>,
) -> Result<Json<ItemsResponse>, AppError> {
    tracing::info!("Downloading episodes of {}", payload.feed_url);

    let list = state.rssfeed_service
        .list_episodes(&payload.feed_url, 20)
        .await?;

    Ok(Json(ItemsResponse {
        total: list.len(),
        items: list,
    }))
}

pub async fn save_episode(
    state: State<AppState>,
    Json(payload): Json<FeedRequest>,
) -> Result<Json<Value>, AppError> {
    // todo check database if not in database

    let guid = payload.guid.ok_or_else(|| AppError::InvalidInput("GUID missing from payload".into()))?;

    tracing::info!("Downloading episodes of {} with id {}", payload.feed_url, guid);

    let save_path = "/app/downloads";

    let episode_path = state.rssfeed_service
        .download_episode(&payload.feed_url, &guid, &save_path)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "file_path": episode_path
    })))
}