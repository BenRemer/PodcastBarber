use crate::error::AppError;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct TranscribeJob {
    pub file_name: String,
    pub content_type: String,
    pub data: bytes::Bytes,
}

pub struct TranscribeResult {
    pub transcription: Option<Value>,
    pub error: Option<AppError>,
}
