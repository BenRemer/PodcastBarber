use async_trait::async_trait;
use barber_api::error::AppError;
use barber_api::services::detection::ProcessedSegment;
use barber_api::services::editor::Editor;
use std::path::{Path, PathBuf};

pub struct MockEditor;

#[async_trait]
impl Editor for MockEditor {
    async fn remove_ads(
        &self,
        _episode_path: &Path,
        _detection: &Vec<ProcessedSegment>,
        _beep: &[u8],
    ) -> Result<PathBuf, AppError> {
        // Just return a fake successful path!
        Ok(PathBuf::from("/tmp/mock_clean.mp3"))
    }
}
