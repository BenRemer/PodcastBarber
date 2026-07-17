use crate::error::AppError;
use crate::services::detection::manual::detect_ads;
use crate::services::detection::types::ProcessedSegment;
use serde_json::Value;

pub struct DetectionCore {}

impl DetectionCore {
    pub fn new() -> Self {
        Self {}
    }

    pub fn detect_ads(&self, transcript: &Value) -> Result<Vec<ProcessedSegment>, AppError> {
        // Manual detection
        let scored_segments = detect_ads(transcript)?;
        Ok(scored_segments)
    }
}
