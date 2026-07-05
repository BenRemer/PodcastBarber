mod feeds;
pub(crate) mod models;

use axum::{routing::{post, get}, Router};
use crate::state::AppState;

pub fn rss_router() -> Router<AppState> {
    Router::new()
        // .route("/podcasts/download", post(podcasts::handle_podcast_download))
        .route("/feeds/newest", post(feeds::get_latest_mp3))
        .route("/feeds/new", post(feeds::list_episodes))
}