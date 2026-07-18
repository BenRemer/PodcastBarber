use crate::services::detection::manual::config::ScoreConfig;

pub(crate) fn score_segment(config: &ScoreConfig, text: &str, duration_seconds: f64) -> i32 {
    let mut score = 0;
    let mut keyword_hits = 0;

    let text_lower = text.to_lowercase();
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
