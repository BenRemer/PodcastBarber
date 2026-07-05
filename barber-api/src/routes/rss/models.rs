use serde::Serialize;

#[derive(Serialize)]
pub struct EpisodesResponse {
    pub items: Vec<EpisodeItem>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct EpisodeItem {
    pub guid: String,
    pub title: String,
    pub audio_url: String,
    pub publish_date: Option<String>,
}