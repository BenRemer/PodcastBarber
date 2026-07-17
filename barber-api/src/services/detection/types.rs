use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct DetectionJob {
    pub tracking_id: Uuid,
}

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
