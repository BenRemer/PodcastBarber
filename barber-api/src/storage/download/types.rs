use crate::error::AppError;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub tracking_id: Uuid,
    pub audio_url: String,
    pub folder_name: String,
    pub guid: String,
}

#[derive(Debug)]
pub struct DownloadResult {
    pub tracking_id: Uuid,
    pub status: Result<PathBuf, AppError>,
}
