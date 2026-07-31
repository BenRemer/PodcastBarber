use schemars::JsonSchema;
use serde::Serialize;
use crate::error::AppError;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct DetectionJob {
    pub episode_id: Uuid,
}

#[derive(Debug)]
pub struct DetectionResult {
    pub episode_id: Uuid,
    pub error: Option<AppError>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProcessedSegment {
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
    pub ad_score: i32,
    pub is_ad: bool,
}

#[derive(Debug)]
pub struct Detection {
    pub episode_id: Uuid,
    pub segments: Vec<ProcessedSegment>,
}

pub struct Segment {
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct TranscriptChunk {
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub embedding: Vec<f32>,
}
