use crate::services::detection::manual::constants::{AD_KEYWORDS, OUTRO_KEYWORDS};
use crate::services::detection::manual::types::{SegmentBoundary, TranscriptChunk};
use crate::services::detection::types::ProcessedSegment;

#[derive(Debug, Clone)]
pub(crate) struct ScoreConfig {
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

pub(crate) fn classify_segments(
    config: &ScoreConfig,
    chunks: &[TranscriptChunk],
    boundaries: &[SegmentBoundary],
    total_duration: f64,
) -> Vec<ProcessedSegment> {
    let mut segments = Vec::new();
    let mut current_start = 0.0;

    // Add a "virtual" boundary at the very end of the podcast to catch the final segment
    let mut all_boundary_times: Vec<f64> = boundaries.iter().map(|b| b.timestamp).collect();
    all_boundary_times.push(total_duration);

    for boundary_time in all_boundary_times {
        let mut segment_text = String::new();

        for chunk in chunks {
            if chunk.start_time < boundary_time && chunk.end_time > current_start {
                segment_text.push_str(&chunk.text);
                segment_text.push(' ');
            }
        }

        // Score it
        let duration = boundary_time - current_start;
        let ad_score = score_segment(&config, &segment_text, duration);

        segments.push(ProcessedSegment {
            start_time: current_start,
            end_time: boundary_time,
            text: segment_text.trim().to_string(),
            ad_score,
            is_ad: ad_score >= config.ad_threshold,
        });

        // Move the start bracket forward for the next segment
        current_start = boundary_time;
    }

    segments
}

fn score_segment(config: &ScoreConfig, text: &str, duration_seconds: f64) -> i32 {
    let mut score = 0;
    let text_lower = text.to_lowercase();
    let mut keyword_hits = 0;

    for keyword in config.keywords {
        let padded_keyword = format!(" {} ", keyword);
        let occurrences = text_lower.matches(&padded_keyword).count() as i32;

        let starts_with = i32::from(text_lower.starts_with(keyword));

        let total_hits = occurrences + starts_with;
        keyword_hits += total_hits;
        score += total_hits * config.keyword_weight;
    }

    if duration_seconds >= config.preferred_min_duration_secs
        && duration_seconds <= config.preferred_max_duration_secs
    {
        score += config.preferred_duration_bonus;
    } else if duration_seconds > config.long_segment_threshold_secs {
        score -= config.long_segment_penalty;
    } else if duration_seconds < config.short_segment_threshold_secs && keyword_hits == 0 {
        score -= config.short_segment_penalty;
    }

    if config
        .outro_phrases
        .iter()
        .any(|phrase| text_lower.contains(phrase))
    {
        score -= config.outro_penalty;
    }

    score
}
