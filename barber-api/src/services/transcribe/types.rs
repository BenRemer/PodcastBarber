use crate::error::AppError;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TranscribeJob {
    pub episode_id: Uuid,
    pub file_path: PathBuf,
}

pub struct TranscribeResult {
    pub episode_id: Uuid,
    pub error: Option<AppError>,
}

// todo move to enum for better state management
// pub enum TranscribeResult {
//     Success { episode_id: Uuid },
//     Failure {
//         episode_id: Uuid,
//         error: AppError,
//     },
// }
