use aide::axum::ApiRouter;
use aide::axum::routing::{delete, get, post};
use crate::models::api::{EpisodeItem, EpisodeQuery, EpisodeRequest};
use crate::models::episode::Episode;
use crate::{error::AppError, state::AppState};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use axum::extract::{Path, Query};
use uuid::Uuid;

pub fn episodes_router() -> ApiRouter<AppState> {
    // /api/podcasts/{id}/episodes
    ApiRouter::new()
        .api_route("/", get(list_episodes))
        .api_route("/", post(save_episode))
        .api_route("/subscribed", get(list_subscribed_episodes))
        .api_route("/{episode_id}/unsubscribe", delete(remove_episode))
}

pub async fn list_episodes(
    State(state): State<AppState>,
    Path(podcast_id): Path<Uuid>,
    Query(query): Query<EpisodeQuery>,
) -> Result<(StatusCode, Json<Vec<EpisodeItem>>), AppError> {
    tracing::info!("Listing episodes for: {:?}", podcast_id);

    let podcast = state
        .podcast_service
        .get_podcast_by_id(&podcast_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let episodes = state
        .rss_service
        .list_episodes(&podcast.feed_url, query.limit)
        .await?;

    Ok((StatusCode::OK, Json(episodes)))
}

pub async fn list_subscribed_episodes(
    State(state): State<AppState>,
    Path(podcast_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<Episode>>), AppError> {
    tracing::info!("Listing episodes for: {:?}", podcast_id);

    let podcast = state
        .podcast_service
        .get_podcast_by_id(&podcast_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let episodes = state
        .episode_service
        .get_episodes_by_podcast(&podcast.id)
        .await?;

    Ok((StatusCode::OK, Json(episodes)))
}

pub async fn save_episode(
    State(state): State<AppState>,
    Path(podcast_id): Path<Uuid>,
    Json(payload): Json<EpisodeRequest>,
) -> Result<(StatusCode, Json<Episode>), AppError> {
    if !state
        .podcast_service
        .is_subscribed_by_id(&podcast_id)
        .await?
    {
        return Err(AppError::NotFound);
    }

    let podcast = state
        .podcast_service
        .get_podcast_by_id(&podcast_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("Podcast is missing from the database".into()))?;

    tracing::info!(
        "Downloading episode of {} with id {}",
        podcast.title,
        payload.guid
    );

    let channel = state
        .rss_service
        .construct_rss_channel(&podcast.feed_url)
        .await?;
    let episode_metadata = state
        .rss_service
        .get_episode_metadata(&channel, &podcast.feed_url, &payload.guid)
        .await?;

    let episode = episode_metadata.into_pending_episode(podcast_id);

    let episode = state
        .episode_service
        .queue_episode_download(podcast, episode)
        .await?;

    Ok((StatusCode::CREATED, Json(episode)))
}

pub async fn remove_episode(
    State(state): State<AppState>,
    Path((podcast_id, episode_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    tracing::info!("Deleting episode of {} with id {}", podcast_id, episode_id);

    if !state
        .podcast_service
        .is_subscribed_by_id(&podcast_id)
        .await?
    {
        return Err(AppError::NotFound);
    }

    let episode = state
        .episode_service
        .get(&episode_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("Podcast is missing from the database".into()))?;

    state.episode_service.delete_episode(episode).await?;
    // Returns 204 No Content
    Ok(StatusCode::NO_CONTENT)
}
