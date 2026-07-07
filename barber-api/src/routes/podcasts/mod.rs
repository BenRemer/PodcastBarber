mod podcasts;

use axum::{routing::{post, get}, Router};
use crate::state::AppState;

pub fn podcasts_router() -> Router<AppState> {
    Router::new()
        .route("/", get(podcasts::list_subscribed_podcasts))
        .route("/", post(podcasts::subscribe_to_podcast))
        .route("/download", post(podcasts::save_episode))
}