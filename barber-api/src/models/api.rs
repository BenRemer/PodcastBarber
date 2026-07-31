use schemars::JsonSchema;
pub(crate) use crate::models::episode::EpisodeItem;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
pub struct PodcastRequest {
    pub feed_url: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct EpisodeQuery {
    pub limit: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EpisodeRequest {
    pub guid: String,
}

#[derive(Serialize, JsonSchema)]
pub struct EpisodesResponse {
    pub items: Vec<EpisodeItem>,
    pub total: usize,
}
