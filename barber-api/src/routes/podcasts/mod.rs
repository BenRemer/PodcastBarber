mod podcasts;

use axum::{routing::{post}, Router};
use crate::state::AppState;

pub fn podcasts_router() -> Router<AppState> {
    Router::new()
        .route("/podcasts", post(podcasts::subscribe_to_podcast))
        .route("/podcasts/download", post(podcasts::save_episode))
}