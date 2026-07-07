use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum EpisodeState {
    Pending,
    Downloaded,
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
