use axum::{routing::get, routing::post, Router};

pub mod audio;
mod ping;
mod podcasts;
mod feeds;

use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/ping", get(ping::handle_ping))
        .route("/transcribe", post(audio::handle_upload))
        .route("/rss/feeds/list", post(feeds::list_episodes))
        .nest("/podcasts", podcasts::podcasts_router())
        .with_state(state)
}