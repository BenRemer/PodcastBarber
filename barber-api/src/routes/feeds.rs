use axum::{extract::State, Json};
use axum::http::StatusCode;
use serde::Deserialize;
use crate::{state::AppState, error::AppError};
use crate::models::api::EpisodesResponse;

#[derive(Deserialize)]
pub struct FeedRequest {
    pub feed_url: String,
    pub size: Option<usize>,
}

pub async fn list_episodes(
    State(state): State<AppState>,
    Json(payload): Json<FeedRequest>,
) -> Result<(StatusCode, Json<EpisodesResponse>), AppError> {
    let size = payload.size.unwrap_or(20);
    tracing::info!("Listing {} episodes of {}",size, payload.feed_url);

    let list = state.rss_service
        .list_episodes(&payload.feed_url, Some(size))
        .await?;

    Ok((StatusCode::OK ,
        Json(EpisodesResponse {
            total: list.len(),
            items: list,
    })))
}
