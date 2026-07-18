use crate::services::detection::manual::constants::{AD_KEYWORDS, OUTRO_KEYWORDS};

#[derive(Debug, Clone)]
pub struct ScoreConfig {
    pub keywords: &'static [&'static str],
    pub outro_phrases: &'static [&'static str],
    pub keyword_weight: i32,
    pub preferred_min_duration_secs: f64,
    pub preferred_max_duration_secs: f64,
    pub preferred_duration_bonus: i32,
    pub long_segment_threshold_secs: f64,
    pub long_segment_penalty: i32,
    pub short_segment_threshold_secs: f64,
    pub short_segment_penalty: i32,
    pub outro_penalty: i32,
    pub ad_threshold: i32,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            keywords: AD_KEYWORDS,
            outro_phrases: OUTRO_KEYWORDS,
            keyword_weight: 15,
            preferred_min_duration_secs: 30.0,
            preferred_max_duration_secs: 120.0,
            preferred_duration_bonus: 15,
            long_segment_threshold_secs: 180.0,
            long_segment_penalty: 100,
            short_segment_threshold_secs: 15.0,
            short_segment_penalty: 20,
            outro_penalty: 25,
            ad_threshold: 25,
        }
    }
}
