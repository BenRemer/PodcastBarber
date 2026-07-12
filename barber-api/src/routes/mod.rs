use axum::{Router, routing::get, routing::post};

pub mod audio;
mod episodes;
mod feeds;
mod ping;
mod podcast;

use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/ping", get(ping::handle_ping))
        .route("/transcribe", post(audio::handle_episode_transcribe))
        .route("/rss/list", post(feeds::list_episodes))
        .nest("/podcasts", podcast::podcasts_router())
        .nest("/podcasts/{podcast}/episodes", episodes::episodes_router())
        .with_state(state)
}
