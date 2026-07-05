use axum::{routing::post, routing::get, Router};

pub mod audio;
mod ping;
pub(crate) mod rss;

use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/ping", get(ping::handle_ping))
        .route("/transcribe", post(audio::handle_upload))
        .nest("/rss", rss::rss_router())
        .with_state(state)
}