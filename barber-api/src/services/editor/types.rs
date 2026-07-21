use crate::error::AppError;
use crate::services::detection::ProcessedSegment;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EditorJob {
    pub episode_id: Uuid,
    pub episode_path: PathBuf,
    pub segments: Vec<ProcessedSegment>,
}

pub enum EditorResult {
    Success { episode_id: Uuid, path: PathBuf },
    Failure { episode_id: Uuid, error: AppError },
}

impl EditorResult {
    /// Returns true if the result was a success
    pub fn is_success(&self) -> bool {
        matches!(self, EditorResult::Success { .. })
    }

    /// Returns true if the result was a failure
    pub fn is_failure(&self) -> bool {
        matches!(self, EditorResult::Failure { .. })
    }

    /// A handy helper to always get the episode ID, regardless of outcome
    pub fn episode_id(&self) -> Uuid {
        match self {
            EditorResult::Success { episode_id, .. } => *episode_id,
            EditorResult::Failure { episode_id, .. } => *episode_id,
        }
    }
}
