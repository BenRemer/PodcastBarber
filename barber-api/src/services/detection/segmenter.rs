use crate::services::detection::types::Segment;
use crate::services::detection::types::TranscriptChunk;

pub fn create_segments(chunks: &[TranscriptChunk], boundaries: &[f64]) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut current_start = 0.0;

    let mut boundary_times = boundaries.to_vec();
    boundary_times.push(chunks.last().map_or(0.0, |c| c.end_time));

    for boundary_time in boundary_times {
        let text = chunks
            .iter()
            .filter(|chunk| chunk.start_time < boundary_time && chunk.end_time > current_start)
            .map(|chunk| chunk.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        segments.push(Segment {
            start_time: current_start,
            end_time: boundary_time,
            text,
        });

        current_start = boundary_time;
    }

    segments
}
