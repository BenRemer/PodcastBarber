use axum::{extract::State, Json};
use serde::Deserialize;
use crate::{state::AppState, error::AppError};
use crate::models::api::EpisodesResponse;

#[derive(Deserialize)]
pub struct FeedRequest {
    pub feed_url: String,
    pub guid: Option<String>,
    pub size: Option<usize>,
}

pub async fn list_episodes(
    State(state): State<AppState>,
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
