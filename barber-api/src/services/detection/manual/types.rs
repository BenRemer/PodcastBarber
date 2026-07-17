#[derive(Debug, Clone)]
pub(crate) struct TranscriptChunk {
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub embedding: Vec<f32>,
}

#[derive(Debug)]
pub(crate) struct SegmentBoundary {
    pub chunk_index: usize, // todo remove?
    pub timestamp: f64,
    pub depth_score: f32,
}
