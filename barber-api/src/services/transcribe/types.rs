use crate::error::AppError;
use crate::models::transcript::Transcript;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TranscribeJob {
    pub episode_id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub data: bytes::Bytes,
}

pub struct TranscribeResult {
    pub episode_id: Uuid,
    pub transcription: Option<Transcript>,
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
