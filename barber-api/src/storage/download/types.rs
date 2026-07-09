use std::path::PathBuf;
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub episode_id: Uuid,
    pub audio_url: String,
    pub podcast_title: String,
    pub guid: String,
}

#[derive(Debug)]
pub struct DownloadResult {
    pub id: Uuid,
    pub status: Result<PathBuf, AppError>,
}
