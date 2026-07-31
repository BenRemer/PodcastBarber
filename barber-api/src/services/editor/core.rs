use crate::error::AppError;
use crate::services::detection::ProcessedSegment;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Clone)]
pub struct EditorCore {
    pub base_dir: PathBuf,
}

impl EditorCore {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Helper to find continuous blocks of audio we want to keep
    fn calculate_keep_blocks(segments: &[ProcessedSegment]) -> Vec<(f64, f64)> {
        // Collect references and sort them chronologically
        let mut sorted_segments = segments.to_vec();
        sorted_segments.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

        let mut keep_blocks = Vec::new();
        let mut current_start = None;
        let mut current_end = 0.0;

        for seg in sorted_segments {
            if !seg.is_ad {
                // If this is the start of a new safe block, mark the start time
                if current_start.is_none() {
                    current_start = Some(seg.start_time);
                }
                // Push the end time forward
                current_end = seg.end_time;
            } else {
                // We hit an ad. If we were tracking a safe block, close it and save it.
                if let Some(start) = current_start {
                    keep_blocks.push((start, current_end));
                    current_start = None; // Reset for the next safe block
                }
            }
        }

        // If the file ends on a safe block, make sure we save the final chunk
        if let Some(start) = current_start {
            keep_blocks.push((start, current_end));
        }

        keep_blocks
    }
}

#[async_trait]
pub trait Editor: Send + Sync {
    async fn remove_ads(
        &self,
        episode_path: &Path,
        detection: &Vec<ProcessedSegment>,
        beep: &[u8],
    ) -> Result<PathBuf, AppError>;
}

#[async_trait]
impl Editor for EditorCore {
    async fn remove_ads(
        &self,
        episode_path: &Path,
        detection: &Vec<ProcessedSegment>,
        beep: &[u8],
    ) -> Result<PathBuf, AppError> {
        // Group the non-ad segments into continuous blocks to KEEP
        let keep_blocks = Self::calculate_keep_blocks(&detection);

        if keep_blocks.is_empty() {
            return Err(AppError::InternalServerError(
                "Entire file marked as ads".into(),
            ));
        }

        // Setup file paths
        let dir = episode_path.parent().unwrap();
        let file_stem = episode_path.file_stem().unwrap().to_string_lossy();
        let ext = episode_path
            .extension()
            .unwrap_or_default()
            .to_string_lossy();

        let output_path = dir.join(format!("{}_clean.{}", file_stem, ext));
        let list_path = dir.join(format!("{}_concat.txt", Uuid::new_v4()));

        // Ensure the assets folder exists
        let assets_dir = self.base_dir.join("assets");
        if !assets_dir.exists() {
            fs::create_dir_all(&assets_dir).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to create assets dir: {}", e).into())
            })?;
        }

        // Write the beep bytes to disk if they aren't already there
        let boop_path = assets_dir.join("boop.mp3");
        if !boop_path.exists() {
            fs::write(&boop_path, beep).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to write boop file: {}", e).into())
            })?;
        }

        // Generate the FFmpeg concat instructions
        let mut concat_file_content = String::new();

        // Escape paths for FFmpeg text files
        let safe_input_path = episode_path.to_string_lossy().replace("'", "'\\''");
        let safe_boop_path = boop_path.to_string_lossy().replace("'", "'\\''");

        // Iterate with .enumerate() so we know which block we are on
        for (index, (start, end)) in keep_blocks.iter().enumerate() {
            // Write the safe block
            concat_file_content.push_str(&format!("file '{}'\n", safe_input_path));
            concat_file_content.push_str(&format!("inpoint {:.3}\n", start));
            concat_file_content.push_str(&format!("outpoint {:.3}\n", end));

            // If this is NOT the final block, an ad was removed here. Insert the boop!
            if index < keep_blocks.len() - 1 {
                concat_file_content.push_str(&format!("file '{}'\n", safe_boop_path));
            }
        }

        fs::write(&list_path, concat_file_content)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to write concat file: {}", e).into())
            })?;

        // Run FFmpeg to splice the file without re-encoding (-c copy)
        let output = Command::new("ffmpeg")
            .arg("-y") // Overwrite output if it exists
            .arg("-f")
            .arg("concat") // Use the concat demuxer
            .arg("-safe")
            .arg("0") // Allow absolute paths in the txt file
            .arg("-i")
            .arg(&list_path) // The text file we just created
            .arg("-c")
            .arg("copy") // COPY the streams (No re-encoding!)
            .arg(&output_path)
            .output()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to execute ffmpeg: {}", e).into())
            })?;

        // Cleanup the temporary text file
        let _ = fs::remove_file(&list_path).await;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("FFmpeg failed: {}", stderr);
            // clean up failed garbage
            let _ = fs::remove_file(&output_path).await;
            return Err(AppError::InternalServerError(
                "FFmpeg processing failed".into(),
            ));
        }

        // overwrite existing file with new edited file
        fs::rename(&output_path, episode_path).await.map_err(|e| {
            AppError::InternalServerError(
                format!("Failed to overwrite original file: {}", e).into(),
            )
        })?;
        tracing::info!("Overwrote original file");

        Ok(output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_keep_blocks() {
        // Create a fake transcript with ads and non-ads mixed up (out of chronological order)
        let segments = vec![
            ProcessedSegment {
                start_time: 20.0,
                end_time: 30.0,
                text: "".into(),
                ad_score: 0,
                is_ad: false,
            }, // Safe
            ProcessedSegment {
                start_time: 10.0,
                end_time: 20.0,
                text: "".into(),
                ad_score: 100,
                is_ad: true,
            }, // AD!
            ProcessedSegment {
                start_time: 0.0,
                end_time: 10.0,
                text: "".into(),
                ad_score: 0,
                is_ad: false,
            }, // Safe
            ProcessedSegment {
                start_time: 30.0,
                end_time: 40.0,
                text: "".into(),
                ad_score: 0,
                is_ad: false,
            }, // Safe
        ];

        // Run the private function
        let keep_blocks = EditorCore::calculate_keep_blocks(&segments);

        // Expected behavior:
        // - Sorts chronologically
        // - Merges the 0-10 block
        // - Skips the 10-20 ad block
        // - Merges the 20-30 and 30-40 blocks into a single 20-40 block
        assert_eq!(keep_blocks.len(), 2);
        assert_eq!(keep_blocks[0], (0.0, 10.0));
        assert_eq!(keep_blocks[1], (20.0, 40.0));
    }

    #[test]
    fn test_calculate_keep_blocks_all_ads() {
        let segments = vec![ProcessedSegment {
            start_time: 0.0,
            end_time: 10.0,
            text: "".into(),
            ad_score: 100,
            is_ad: true,
        }];
        let keep_blocks = EditorCore::calculate_keep_blocks(&segments);
        assert!(keep_blocks.is_empty());
    }
}
