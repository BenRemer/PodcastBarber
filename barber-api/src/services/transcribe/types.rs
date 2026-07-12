use crate::error::AppError;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TranscribeJob {
    pub tracking_id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub data: bytes::Bytes,
}

pub struct TranscribeResult {
    pub tracking_id: Uuid,
    pub transcription: Option<Value>,
    pub error: Option<AppError>,
}
