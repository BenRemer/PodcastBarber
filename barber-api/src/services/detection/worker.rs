use crate::error::AppError;
use crate::services::detection::Detection;
use crate::services::detection::chunker::TranscriptChunker;
use crate::services::detection::core::Detector;
use crate::services::detection::types::{DetectionJob, DetectionResult};
use crate::storage::repository::detection::DetectionStore;
use crate::storage::repository::transcript::TranscriptStore;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};

pub struct DetectionWorker {
    pub transcript_repository: Arc<dyn TranscriptStore>,
    pub detection_repository: Arc<dyn DetectionStore>,
    pub core: Arc<dyn Detector>,
    pub chunker: Arc<dyn TranscriptChunker>,
    pub concurrency_limit: usize,
    pub job_queue: mpsc::Receiver<DetectionJob>,
    pub callback: mpsc::Sender<DetectionResult>, // callback to audio processor
}

impl DetectionWorker {
    pub async fn run(mut self) {
        tracing::info!("Starting detection worker...");

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));

        while let Some(job) = self.job_queue.recv().await {
            tracing::info!("Received job: {:?}", job);

            let chunker_clone = self.chunker.clone();
            let core_clone = self.core.clone();
            let callback_clone = self.callback.clone();
            let transcript_store = self.transcript_repository.clone();
            let detection_store = self.detection_repository.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            tokio::spawn(async move {
                match Self::process(
                    &job,
                    chunker_clone,
                    core_clone,
                    detection_store,
                    transcript_store,
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("Detection successful!");
                        let _ = callback_clone
                            .send(DetectionResult {
                                episode_id: job.episode_id,
                                error: None,
                            })
                            .await;
                    }
                    Err(e) => {
                        tracing::error!("Detection job failed: {:?}", e);
                        let _ = callback_clone
                            .send(DetectionResult {
                                episode_id: job.episode_id,
                                error: Some(e),
                            })
                            .await;
                    }
                }
                drop(permit);
            });
        }
        tracing::info!("Detection worker ended.");
    }

    async fn process(
        job: &DetectionJob,
        chunker: Arc<dyn TranscriptChunker>,
        core: Arc<dyn Detector>,
        detection_store: Arc<dyn DetectionStore>,
        transcript_store: Arc<dyn TranscriptStore>,
    ) -> Result<(), AppError> {
        let transcript = transcript_store
            .get_by_episode_id(&job.episode_id)
            .await?
            .ok_or_else(|| {
                AppError::InternalServerError("Transcript not found for episode".into())
            })?;

        let chunks = chunker.chunk(&transcript.data, 5.0)?;
        let processed = core.detect_ads(&chunks);
        let detection = Detection {
            episode_id: job.episode_id,
            segments: processed,
        };

        Ok(detection_store.upsert(detection).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transcript::Transcript;
    use crate::services::detection::{ProcessedSegment, TranscriptChunk};
    use async_trait::async_trait;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time::Instant;
    use uuid::Uuid;

    struct MockTranscriptStore;
    #[async_trait]
    impl TranscriptStore for MockTranscriptStore {
        async fn upsert(&self, transcript: Transcript) -> Result<Transcript, AppError> {
            Ok(transcript)
        }
        async fn get_by_episode_id(
            &self,
            episode_id: &Uuid,
        ) -> Result<Option<Transcript>, AppError> {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok(Some(Transcript {
                id: Uuid::new_v4(),
                episode_id: *episode_id,
                data: serde_json::json!({"text": "fake transcript data"}),
            }))
        }
    }

    struct MockChunker;
    impl TranscriptChunker for MockChunker {
        fn chunk(
            &self,
            _data: &serde_json::Value,
            _duration: f64,
        ) -> Result<Vec<TranscriptChunk>, AppError> {
            Ok(vec![])
        }
    }

    struct MockDetectionCore;
    impl Detector for MockDetectionCore {
        fn detect_ads(&self, _chunks: &[TranscriptChunk]) -> Vec<ProcessedSegment> {
            vec![]
        }
    }

    struct MockDetectionStore;
    #[async_trait]
    impl DetectionStore for MockDetectionStore {
        async fn upsert(&self, _detection: Detection) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_detection_by_episode(
            &self,
            _episode_id: &Uuid,
        ) -> Result<Option<Detection>, AppError> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_detection_worker_processes_concurrently() {
        let (job_tx, job_rx) = mpsc::channel(10);
        let (result_tx, mut result_rx) = mpsc::channel(10);

        let worker = DetectionWorker {
            transcript_repository: Arc::new(MockTranscriptStore) as Arc<dyn TranscriptStore>,
            detection_repository: Arc::new(MockDetectionStore) as Arc<dyn DetectionStore>,
            core: Arc::new(MockDetectionCore),
            chunker: Arc::new(MockChunker),
            job_queue: job_rx,
            callback: result_tx,
            concurrency_limit: 3,
        };

        tokio::spawn(worker.run());

        for _ in 0..3 {
            let job = DetectionJob {
                episode_id: Uuid::new_v4(),
            };
            job_tx.send(job).await.unwrap();
        }

        // Drop the sender so the worker terminates after processing the queue
        drop(job_tx);

        let start_time = Instant::now();
        let mut results = Vec::new();

        for _ in 0..3 {
            if let Some(res) = result_rx.recv().await {
                results.push(res);
            }
        }

        let elapsed = start_time.elapsed();

        assert_eq!(results.len(), 3, "Worker did not process all jobs");
        for res in &results {
            assert!(res.error.is_none(), "Detection job failed: {:?}", res.error);
        }

        // If it ran sequentially, the 200ms DB delay would cause this to take 600ms total.
        // Because it's fully concurrent, they all wait in parallel, taking ~200ms total.
        assert!(
            elapsed < Duration::from_millis(400),
            "Worker is running sequentially! It took {:?}",
            elapsed
        );
    }
}
