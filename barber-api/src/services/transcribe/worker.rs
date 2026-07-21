use crate::models::transcript::Transcript;
use crate::services::transcribe::core::Transcriber;
use crate::services::transcribe::types::{TranscribeJob, TranscribeResult};
use crate::storage::repository::transcript::TranscriptStore;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use uuid::Uuid;

pub struct TranscribeWorker {
    pub(crate) transcript_repository: Arc<dyn TranscriptStore>,
    pub(crate) core: Arc<dyn Transcriber>,
    pub(crate) queue_receive: mpsc::Receiver<TranscribeJob>,
    pub(crate) callback: mpsc::Sender<TranscribeResult>,
    pub(crate) concurrency_limit: usize,
}

impl TranscribeWorker {
    pub async fn run(mut self) {
        tracing::info!("Starting Transcribe Worker...");

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));

        while let Some(job) = self.queue_receive.recv().await {
            tracing::info!("Transcribe job received: {:?}", job);

            let core_clone = self.core.clone();
            let callback_clone = self.callback.clone();
            let repo_clone = self.transcript_repository.clone();

            let permit = semaphore.clone().acquire_owned().await.unwrap();
            tokio::spawn(async move {
                let episode_id = job.episode_id;
                let path = job.file_path;

                let Ok(file_bytes) = tokio::fs::read(&path).await else {
                    tracing::info!("Failed to read episode {}", episode_id);
                    return;
                };

                let content_type = infer::get(&file_bytes)
                    .map(|kind| kind.mime_type().to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                let file_name = path.file_name().unwrap().to_string_lossy().into_owned();

                match core_clone
                    .transcribe_audio(file_name, content_type, file_bytes.into())
                    .await
                {
                    Ok(data) => {
                        let transcript = Transcript {
                            id: Uuid::new_v4(),
                            episode_id,
                            data,
                        };
                        // write to repo
                        match repo_clone.upsert(transcript).await {
                            Ok(_) => {
                                let _ = callback_clone
                                    .send(TranscribeResult {
                                        episode_id,
                                        error: None,
                                    })
                                    .await;
                            }
                            Err(e) => {
                                let _ = callback_clone
                                    .send(TranscribeResult {
                                        episode_id,
                                        error: Some(e),
                                    })
                                    .await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = callback_clone
                            .send(TranscribeResult {
                                episode_id,
                                error: Some(e),
                            })
                            .await;
                    }
                }
                drop(permit);
            });
        }
        tracing::info!("Stopping Transcribe Worker.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use async_trait::async_trait;
    use serde_json::{Value, json};
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time::Instant;

    struct MockTranscriber;

    #[async_trait]
    impl Transcriber for MockTranscriber {
        async fn transcribe_audio(
            &self,
            _file_name: String,
            _content_type: String,
            _data: bytes::Bytes,
        ) -> Result<Value, AppError> {
            // Simulate a slow API/GPU transcription call
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok(json!({"text": "mocked transcription text"}))
        }

        async fn check_health(&self) -> Result<Value, AppError> {
            unimplemented!()
        }
    }

    struct MockTranscriptRepo;

    #[async_trait]
    impl TranscriptStore for MockTranscriptRepo {
        async fn upsert(&self, transcript: Transcript) -> Result<Transcript, AppError> {
            Ok(transcript)
        }

        async fn get_by_episode_id(
            &self,
            _episode_id: &Uuid,
        ) -> Result<Option<Transcript>, AppError> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_transcribe_worker_concurrency() {
        let (job_tx, job_rx) = mpsc::channel(10);
        let (result_tx, mut result_rx) = mpsc::channel(10);

        let temp_dir = std::env::temp_dir();
        let mut test_files = Vec::new();

        for i in 0..3 {
            let file_path = temp_dir.join(format!("test_audio_{}.mp3", i));
            // Write a tiny bit of dummy data to the temp file
            tokio::fs::write(&file_path, b"fake audio data")
                .await
                .unwrap();
            test_files.push(file_path);
        }

        let worker = TranscribeWorker {
            core: Arc::new(MockTranscriber),
            queue_receive: job_rx,
            callback: result_tx,
            concurrency_limit: 3,
            transcript_repository: Arc::new(MockTranscriptRepo),
        };

        tokio::spawn(worker.run());

        for path in test_files.iter() {
            let job = TranscribeJob {
                episode_id: Uuid::new_v4(),
                file_path: path.clone(),
            };
            job_tx.send(job).await.unwrap();
        }

        // Drop the sender to trigger graceful shutdown
        drop(job_tx);

        // Measure execution time
        let start_time = Instant::now();
        let mut results = Vec::new();

        for _ in 0..3 {
            if let Some(res) = result_rx.recv().await {
                results.push(res);
            }
        }

        let elapsed = start_time.elapsed();

        // Clean up dummy files from the OS
        for path in test_files {
            let _ = tokio::fs::remove_file(path).await;
        }

        assert_eq!(results.len(), 3, "Worker did not process all jobs");
        for res in &results {
            assert!(
                res.error.is_none(),
                "Transcription failed with error: {:?}",
                res.error
            );
        }
        // If it ran sequentially, 3 * 200ms = 600ms.
        // If it finishes in under 400ms, Tokio processed them simultaneously!
        assert!(
            elapsed < Duration::from_millis(400),
            "Worker is running sequentially! It took {:?}",
            elapsed
        );
    }
}
