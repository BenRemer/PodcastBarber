use crate::services::detection::manual::{ManualClassifier, ScoreConfig};
use crate::services::detection::math::find_segment_boundaries;
use crate::services::detection::segmenter::create_segments;
use crate::services::detection::types::{ProcessedSegment, TranscriptChunk};

pub trait Detector: Send + Sync {
    fn detect_ads(&self, chunks: &[TranscriptChunk]) -> Vec<ProcessedSegment>;
}

impl Detector for DetectionCore {
    fn detect_ads(&self, chunks: &[TranscriptChunk]) -> Vec<ProcessedSegment> {
        let boundaries = find_segment_boundaries(chunks, self.config.boundary_size);

        let segments = create_segments(chunks, &boundaries);

        // Manual processing
        let processed_segments = segments
            .into_iter()
            .map(|segment| self.manual_classifier.classify(segment))
            .collect();

        processed_segments
    }
}

#[derive(Debug, Clone)]
pub struct DetectionConfig {
    pub boundary_size: usize,
    pub score: ScoreConfig,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            boundary_size: 5,
            score: ScoreConfig::default(),
        }
    }
}

pub struct DetectionCore {
    config: DetectionConfig,
    manual_classifier: ManualClassifier,
}

impl DetectionCore {
    pub fn new(config: DetectionConfig) -> Self {
        let manual_classifier = ManualClassifier::new(config.score.clone());

        Self {
            config,
            manual_classifier,
        }
    }
}
