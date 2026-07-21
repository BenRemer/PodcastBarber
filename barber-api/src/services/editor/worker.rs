use crate::services::editor::core::Editor;
use crate::services::editor::types::{EditorJob, EditorResult};
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};

pub struct EditorWorker {
    pub core: Arc<dyn Editor>,
    pub beep_audio: &'static [u8],
    pub input_queue: mpsc::Receiver<EditorJob>,
    pub callback: mpsc::Sender<EditorResult>, // audio coordinator callback
    pub concurrency_limit: usize,
}

impl EditorWorker {
    pub async fn run(mut self) {
        tracing::info!("AudioEditor background worker starting...");

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));

        while let Some(job) = self.input_queue.recv().await {
            tracing::info!("EditorWorker received job: episode: {}", job.episode_id);

            let core_clone = self.core.clone();
            let callback_clone = self.callback.clone();

            // Wait for an available slot before spawning.
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            tokio::spawn(async move {
                match core_clone
                    .remove_ads(&job.episode_path, &job.segments, self.beep_audio)
                    .await
                {
                    Ok(path) => {
                        let _ = callback_clone
                            .send(EditorResult::Success {
                                episode_id: job.episode_id,
                                path,
                            })
                            .await;
                    }
                    Err(e) => {
                        let _ = callback_clone
                            .send(EditorResult::Failure {
                                episode_id: job.episode_id,
                                error: e,
                            })
                            .await;
                    }
                }
                drop(permit);
            });
        }
        tracing::info!("DownloadWorker background download task shut down.");
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use async_trait::async_trait;
    use uuid::Uuid;
    use crate::error::AppError;
    use crate::services::detection::ProcessedSegment;
    use super::*;

    struct MockEditor;

    #[async_trait]
    impl Editor for MockEditor {
        async fn remove_ads(
            &self,
            _episode_path: &Path,
            _detection: &Vec<ProcessedSegment>,
            _beep: &[u8],
        ) -> Result<PathBuf, AppError> {
            // Simulate processing time
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(PathBuf::from("/fake/clean/path.mp3"))
        }
    }

    #[tokio::test]
    async fn test_editor_worker_processes_jobs() {
        let (job_tx, job_rx) = mpsc::channel(10);
        let (result_tx, mut result_rx) = mpsc::channel(10);

        let worker = EditorWorker {
            core: Arc::new(MockEditor),
            beep_audio: b"fake mp3 bytes",
            input_queue: job_rx,
            callback: result_tx,
            concurrency_limit: 2,
        };

        // Spawn the worker in the background
        tokio::spawn(worker.run());

        let test_episode_id = Uuid::new_v4();

        // Send a mock job to the worker
        let job = EditorJob {
            episode_id: test_episode_id,
            episode_path: PathBuf::from("/fake/input.mp3"),
            segments: vec![],
        };

        job_tx.send(job).await.unwrap();

        // Drop the sender so the worker's `while let` loop naturally closes after finishing
        drop(job_tx);

        // Await the result from the callback channel
        let result = result_rx.recv().await.expect("Worker dropped the result channel");

        // Verify we got a Success event with the correct ID and mocked path
        match result {
            EditorResult::Success { episode_id, path } => {
                assert_eq!(episode_id, test_episode_id);
                assert_eq!(path, PathBuf::from("/fake/clean/path.mp3"));
            }
            EditorResult::Failure { .. } => {
                panic!("Expected Success, got Failure");
            }
        }
    }
}
