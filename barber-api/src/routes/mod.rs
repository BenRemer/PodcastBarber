use aide::axum::ApiRouter;
use aide::axum::routing::{get, post};
use crate::state::AppState;

pub mod audio;
mod episodes;
mod feeds;
mod ping;
mod podcast;

pub fn api_router(state: AppState) -> ApiRouter {
    // /api
    ApiRouter::new()
        .api_route("/ping", get(ping::handle_ping))
        // .route("/transcribe", post(audio::handle_episode_transcribe))
        .api_route("/rss/list", post(feeds::list_episodes))
        .nest("/podcasts", podcast::podcasts_router())
        .nest("/podcasts/{podcast}/episodes", episodes::episodes_router())
        .with_state(state)
}
