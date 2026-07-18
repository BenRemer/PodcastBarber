use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct DetectionJob {
    pub tracking_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct DetectionResult {
    pub tracking_id: Uuid,
}

#[derive(Debug)]
pub struct ProcessedSegment {
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
    pub ad_score: i32,
    pub is_ad: bool,
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
