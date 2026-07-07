use axum::{extract::State, Json, Router};
use axum::extract::Path;
use serde_json::{json, Value};
use axum::routing::{get, post};
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
use crate::models::api::EpisodeRequest;

pub fn episodes_router() -> Router<AppState> {
    // /api/podcasts/:id/episodes
    Router::new()
        .route("/", get(list_saved_episodes))
        .route("/", post(save_episode))
}

pub async fn list_saved_episodes(
    State(state): State<AppState>,
    Path(podcast_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    unimplemented!()
}

pub async fn save_episode(
    State(state): State<AppState>,
    Path(podcast_id): Path<Uuid>,
    Json(payload): Json<EpisodeRequest>,
) -> Result<Json<Value>, AppError> {
    let is_subscribed = state.podcast_service.is_subscribed_by_id(&podcast_id).await?;
    if !is_subscribed {
        return Err(AppError::BadRequest("Podcast is not subscribed".into()));
    }

    let guid = payload.guid
        .ok_or_else(|| AppError::BadRequest("GUID missing from payload".into()))?;

    let podcast = state.podcast_service.get_podcast_by_id(&podcast_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("Podcast is missing from the database".into()))?;

    tracing::info!("Downloading episode of {} with id {}", podcast.title, guid); // todo can ask for title as well?

    let channel = state.rss_service.construct_rss_channel(&podcast.feed_url).await?;
    let episode_metadata = state.rss_service.get_episode_metadata(&channel, &podcast.feed_url,
                                                                  &guid).await?;
    let episode = episode_metadata.into_pending_episode(podcast_id);

    let episode_path = state.episode_service.queue_episode_download(podcast, episode).await?;

    Ok(Json(json!({
        "status": "success",
        "file_path": episode_path
    })))
}
