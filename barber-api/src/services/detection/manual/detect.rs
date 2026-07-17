use crate::error::AppError;
use crate::services::detection::manual::scoring::ScoreConfig;
use crate::services::detection::manual::types::TranscriptChunk;
use crate::services::detection::manual::{math, scoring};
use crate::services::detection::types::ProcessedSegment;
use fastembed::TextEmbedding;
use serde_json::Value;

#[derive(Debug)]
struct RawChunk {
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
}

pub fn detect_ads(transcript: &Value) -> Result<Vec<ProcessedSegment>, AppError> {
    let score_config = ScoreConfig::default();
    let segment_block_size = 3; // todo allow tuning
    let duration = transcript["duration"].as_f64().unwrap_or(0.0);
    let processed_chunks = generate_chunks(transcript, 5.0)?; // todo tuning config
    let boundaries = math::find_segment_boundaries(&processed_chunks, segment_block_size);
    let scored_segments =
        scoring::classify_segments(&score_config, &processed_chunks, &boundaries, duration);

    Ok(scored_segments)
}

fn generate_chunks(transcript: &Value, chunk_size: f64) -> Result<Vec<TranscriptChunk>, AppError> {
    let mut raw_chunks = Vec::new();

    let target_array = transcript
        .as_array()
        .or_else(|| transcript["segments"].as_array())
        .or_else(|| transcript["words"].as_array())
        .or_else(|| transcript["results"].as_array());

    let Some(array) = target_array else {
        return Err(AppError::InternalServerError(
            "Transcript JSON does not contain a valid array of segments.".into(),
        ));
    };

    if array.len() < 2 {
        return Err(AppError::InternalServerError(
            "Transcript contains fewer than 2 segments; cannot create sliding windows.".into(),
        ));
    }

    let mut current_text = String::new();
    let mut current_start = None;
    let mut current_end = 0.0;

    for item in array {
        let text = item["text"].as_str().unwrap_or("");
        let start = item["start"].as_f64().unwrap_or(current_end);
        let end = item["end"].as_f64().unwrap_or(start);

        if current_start.is_none() {
            current_start = Some(start);
        }

        // Ensure proper spacing when joining strings
        if !current_text.is_empty() && !current_text.ends_with(' ') && !text.starts_with(' ') {
            current_text.push(' ');
        }
        current_text.push_str(text.trim());

        current_end = end;

        let duration = current_end - current_start.unwrap();
        let ends_with_punct = text.trim_end().ends_with(&['.', '?', '!'][..]);

        if duration >= chunk_size && ends_with_punct {
            raw_chunks.push(RawChunk {
                text: current_text.clone(),
                start_time: current_start.unwrap(),
                end_time: current_end,
            });

            current_text.clear();
            current_start = None;
        }
    }

    // Catch any leftover text at the very end of the podcast
    if !current_text.is_empty() {
        raw_chunks.push(RawChunk {
            text: current_text,
            start_time: current_start.unwrap_or(0.0),
            end_time: current_end,
        });
    }

    println!("Loading model via FastEmbed...");
    let mut embedder = TextEmbedding::try_new(Default::default())
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let texts: Vec<String> = raw_chunks.iter().map(|c| c.text.clone()).collect();

    let vectors: Vec<Vec<f32>> = embedder
        .embed(texts, None)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Zip our original structs and the new vectors together into our typestate
    let processed_chunks: Vec<TranscriptChunk> = raw_chunks
        .into_iter()
        .zip(vectors.into_iter())
        .map(|(chunk, vector)| TranscriptChunk {
            text: chunk.text,
            start_time: chunk.start_time,
            end_time: chunk.end_time,
            embedding: vector,
        })
        .collect();

    Ok(processed_chunks)
}
