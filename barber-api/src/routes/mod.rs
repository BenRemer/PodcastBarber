use axum::{routing::get, routing::post, Router};

pub mod audio;
mod ping;
mod feeds;
mod podcast;
mod episodes;

use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/ping", get(ping::handle_ping))
        .route("/transcribe", post(audio::handle_upload))
        .route("/rss/list", post(feeds::list_episodes))
        .nest("/podcasts", podcast::podcasts_router())
        .nest("/podcasts/{podcast}/episodes", episodes::episodes_router())
        .with_state(state)
}