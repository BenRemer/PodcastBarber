use axum::{extract::State, Json, Router};
use axum::http::StatusCode;
use axum::routing::{get, post};
use crate::{error::AppError, state::AppState};
use crate::models::api::PodcastRequest;
use crate::models::podcast::Podcast;


pub fn podcasts_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_subscribed_podcasts))
        .route("/", post(subscribe_to_podcast))
        // .route("/download", post(save_episode))
}

pub async fn list_subscribed_podcasts(
    State(state): State<AppState>,
) -> Result<Json<Vec<Podcast>>, AppError> {
    let podcasts = state.podcast_service.list_podcasts().await?;
    Ok(Json(podcasts))
}

// pub async fn save_episode(
//     State(state): State<AppState>,
//     Json(payload): Json<PodcastRequest>,
// ) -> Result<Json<Value>, AppError> {
//     let guid = payload.guid.ok_or_else(|| AppError::BadRequest("GUID missing from payload".into()))?;
//
//     tracing::info!("Downloading episode of {} with id {}", payload.feed_url, guid);
//
//     let episode_path = state.podcast_service
//         .download_episode(&payload.feed_url, &guid)
//         .await?;
//
//     Ok(Json(json!({
//         "status": "success",
//         "file_path": episode_path
//     })))
// }

pub async fn subscribe_to_podcast(
    State(state): State<AppState>,
    Json(payload): Json<PodcastRequest>,
) -> Result<(StatusCode, Json<Podcast>), AppError> {
    let metadata = state.rss_service.fetch_podcast_metadata(&payload.feed_url).await?;
    let podcast = Podcast::from(metadata);

    let saved_podcast  = state.podcast_service
        .subscribe_podcast(podcast)
        .await
        .map_err(|e| {
            tracing::error!("Failed to subscribe to podcast: {:?}", e);
            AppError::InternalServerError("Failure to subscribe".into())
        })?;

    Ok((StatusCode::CREATED, Json(saved_podcast)))
}
