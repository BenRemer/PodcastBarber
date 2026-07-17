use crate::services::detection::manual::types::{SegmentBoundary, TranscriptChunk};

// type SegmentBoundary = f64;

pub(crate) fn find_segment_boundaries(
    chunks: &[TranscriptChunk],
    block_size: usize,
) -> Vec<SegmentBoundary> {
    let k = block_size;

    if chunks.len() < k * 2 {
        return Vec::new();
    }

    let mut similarities = vec![1.0; chunks.len()];

    // Block-Based Similarity Comparison
    for i in k..=(chunks.len() - k) {
        let embedding_dim = chunks[0].embedding.len();

        let mut left_emb = vec![0.0; embedding_dim];
        for j in (i - k)..i {
            for (d, val) in chunks[j].embedding.iter().enumerate() {
                left_emb[d] += val;
            }
        }

        let mut right_emb = vec![0.0; embedding_dim];
        for j in i..(i + k) {
            for (d, val) in chunks[j].embedding.iter().enumerate() {
                right_emb[d] += val;
            }
        }

        similarities[i] = cosine_similarity(&left_emb, &right_emb);
    }

    // Calculate Depth Scores
    let mut depth_scores = vec![0.0; similarities.len()];

    for i in k..=(chunks.len() - k) {
        let current_sim = similarities[i];

        let mut left_peak = current_sim;
        for j in (k..i).rev() {
            if similarities[j] > left_peak {
                left_peak = similarities[j];
            } else {
                break;
            }
        }

        let mut right_peak = current_sim;
        for j in (i + 1)..=(chunks.len() - k) {
            if similarities[j] > right_peak {
                right_peak = similarities[j];
            } else {
                break;
            }
        }

        depth_scores[i] = (left_peak - current_sim) + (right_peak - current_sim);
    }

    // Only calculate threshold based on the valid sliding window area
    let valid_depths: Vec<f32> = depth_scores[k..=(chunks.len() - k)].to_vec();
    if valid_depths.is_empty() {
        return Vec::new();
    }
    let sum_depth: f32 = valid_depths.iter().sum();
    let mean_depth: f32 = sum_depth / valid_depths.len() as f32;

    let variance: f32 = valid_depths
        .iter()
        .map(|value| {
            let diff = *value - mean_depth;
            diff * diff
        })
        .sum::<f32>()
        / valid_depths.len() as f32;
    let std_dev = variance.sqrt();
    let threshold = mean_depth + (std_dev * 0.5);

    // Extract Boundaries
    let mut boundaries = Vec::new();
    for (i, &depth) in depth_scores.iter().enumerate() {
        // Only allow cuts where we actually computed similarity
        if i >= k && i <= chunks.len() - k && depth > threshold {
            boundaries.push(SegmentBoundary {
                chunk_index: i,
                timestamp: chunks[i - 1].end_time, // Cut happens right at the end of the left block
                depth_score: depth,
            });
            // boundaries.push(chunks[i - 1].end_time);
        }
    }

    boundaries
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (val_a, val_b) in a.iter().zip(b.iter()) {
        dot_product += val_a * val_b;
        norm_a += val_a * val_a;
        norm_b += val_b * val_b;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}
