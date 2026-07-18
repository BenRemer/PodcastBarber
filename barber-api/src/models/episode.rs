use crate::utils::generate_episode_uuid;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum EpisodeState {
    Pending,
    Downloaded,
    Transcribed,
    Processing,
    Enhanced,
    Error,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Episode {
    pub id: Uuid,
    pub podcast_id: Uuid,
    pub guid: String,
    pub title: String,
    pub audio_url: String,
    pub local_file_path: Option<String>,
    pub state: EpisodeState,
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
