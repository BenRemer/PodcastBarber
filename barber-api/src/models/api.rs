use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::episode::{Episode, EpisodeState};
use crate::utils::generate_episode_uuid;

#[derive(Serialize)]
pub struct EpisodesResponse {
    pub items: Vec<EpisodeItem>,
    pub total: usize,
}

#[derive(Serialize, Clone)]
pub struct EpisodeItem {
    pub guid: String,
    pub title: String,
    pub audio_url: String,
    pub publish_date: Option<String>,
}

impl EpisodeItem {
    pub fn into_pending_episode(self, podcast_id: Uuid) -> Episode {
        Episode {
            id: generate_episode_uuid(podcast_id, &self.guid),
            podcast_id,
            guid: self.guid,
            title: self.title,
            audio_url: self.audio_url,
            local_file_path: None,
            state: EpisodeState::Pending,
        }
    }
}

// todo clean up locations(should they be in this file)
#[derive(Deserialize)]
pub struct PodcastRequest {
    pub feed_url: String,
    // pub guid: Option<String>,
    // pub size: Option<usize>,
}

#[derive(Deserialize)]
pub struct EpisodeRequest {
    // pub feed_url: String,
    pub guid: Option<String>,
    // pub size: Option<usize>,
}