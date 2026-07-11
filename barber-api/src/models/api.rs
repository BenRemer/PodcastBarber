use serde::{Deserialize, Serialize};
pub(crate) use crate::models::episode::EpisodeItem;

#[derive(Deserialize)]
pub struct PodcastRequest {
    pub feed_url: String,
}

#[derive(Deserialize)]
pub struct EpisodeQuery {
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct EpisodeRequest {
    pub guid: String,
}

#[derive(Serialize)]
pub struct EpisodesResponse {
    pub items: Vec<EpisodeItem>,
    pub total: usize,
}