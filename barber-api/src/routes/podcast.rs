use aide::axum::ApiRouter;
use aide::axum::routing::{get, post};
use crate::models::api::PodcastRequest;
use crate::models::podcast::Podcast;
use crate::{error::AppError, state::AppState};
use axum::http::StatusCode;
use axum::{Json, extract::State};

pub fn podcasts_router() -> ApiRouter<AppState> {
    // /api/podcasts
    ApiRouter::new()
        .api_route("/", get(list_subscribed_podcasts))
        .api_route("/", post(subscribe_to_podcast))
    // .route("/{podcast_id}", get(list_episodes))
}

pub async fn list_subscribed_podcasts(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<Podcast>>), AppError> {
    let podcasts = state.podcast_service.list_podcasts().await?;
    Ok((StatusCode::OK, Json(podcasts)))
}

pub async fn subscribe_to_podcast(
    State(state): State<AppState>,
    Json(payload): Json<PodcastRequest>,
) -> Result<(StatusCode, Json<Podcast>), AppError> {
    let metadata = state
        .rss_service
        .fetch_podcast_metadata(&payload.feed_url)
        .await?;
    let podcast = Podcast::from(metadata);

    let saved_podcast = state
        .podcast_service
        .subscribe_podcast(podcast)
        .await
        .map_err(|e| {
            tracing::error!("Failed to subscribe to podcast: {:?}", e);
            AppError::InternalServerError("Failure to subscribe".into())
        })?;

    Ok((StatusCode::CREATED, Json(saved_podcast)))
}
