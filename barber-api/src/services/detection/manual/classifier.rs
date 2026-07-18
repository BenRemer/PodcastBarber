use crate::services::detection::Segment;
use crate::services::detection::manual::ScoreConfig;
use crate::services::detection::manual::scoring::score_segment;
use crate::services::detection::types::ProcessedSegment;

pub struct ManualClassifier {
    config: ScoreConfig,
}

impl ManualClassifier {
    pub fn new(config: ScoreConfig) -> Self {
        Self { config }
    }

    pub fn classify(&self, segment: Segment) -> ProcessedSegment {
        let duration = segment.end_time - segment.start_time;

        let score = score_segment(&self.config, &segment.text, duration);

        ProcessedSegment {
            start_time: segment.start_time,
            end_time: segment.end_time,
            text: segment.text,
            ad_score: score,
            is_ad: score >= self.config.ad_threshold,
        }
    }
}
