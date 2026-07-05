mod feeds;
pub(crate) mod models;

use axum::{routing::{post}, Router};
use crate::state::AppState;

pub fn rss_router() -> Router<AppState> {
    Router::new()
        // .route("/podcasts/download", post(podcasts::handle_podcast_download))
        .route("/feeds/newest", post(feeds::get_latest_mp3))
        .route("/feeds/list", post(feeds::list_episodes))
        .route("/feeds/download", post(feeds::save_episode))
}