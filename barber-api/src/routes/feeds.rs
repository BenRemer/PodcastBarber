use crate::models::api::EpisodesResponse;
use crate::{error::AppError, state::AppState};
use axum::{Json, extract::State};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct FeedRequest {
    pub feed_url: String,
    pub size: Option<usize>,
}

pub async fn list_episodes(
    State(state): State<AppState>,
    Json(payload): Json<FeedRequest>,
) -> Result<Json<EpisodesResponse>, AppError> {
    let size = payload.size.unwrap_or(20); // todo pass default

    tracing::info!("Listing {} episodes of {}", size, payload.feed_url);

    let list = state
        .rss_service
        .list_episodes(&payload.feed_url, Some(size))
        .await?;

    Ok(Json(EpisodesResponse {
        total: list.len(),
        items: list,
    }))
}
