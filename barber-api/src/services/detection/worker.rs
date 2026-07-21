use crate::error::AppError;
use crate::services::detection::core::DetectionCore;
use crate::services::detection::types::{DetectionJob, DetectionResult};
use crate::services::detection::{Detection, generate_chunks};
use crate::storage::repository::detection::DetectionRepository;
use crate::storage::repository::transcript::TranscriptStore;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct DetectionWorker {
    pub transcript_repository: Arc<dyn TranscriptStore>,
    pub detection_repository: DetectionRepository,
    pub core: DetectionCore,
    pub job_queue: mpsc::Receiver<DetectionJob>,
    pub callback: mpsc::Sender<DetectionResult>, // callback to audio processor
}

impl DetectionWorker {
    pub async fn run(mut self) {
        tracing::info!("Starting detection worker...");
        while let Some(job) = self.job_queue.recv().await {
            tracing::info!("Received job: {:?}", job);
            match self.process(&job).await {
                Ok(_) => {
                    tracing::info!("Detection successful!");
                    let _ = self
                        .callback
                        .send(DetectionResult {
                            episode_id: job.episode_id,
                            error: None,
                        })
                        .await;
                }
                Err(e) => {
                    tracing::error!("Detection job failed: {:?}", e);
                    let _ = self
                        .callback
                        .send(DetectionResult {
                            episode_id: job.episode_id,
                            error: Some(e),
                        })
                        .await;
                }
            }
        }
        tracing::info!("Detection worker ended.");
    }

    async fn process(&self, job: &DetectionJob) -> Result<(), AppError> {
        let transcript = self
            .transcript_repository
            .get_by_episode_id(&job.episode_id)
            .await?
            .ok_or_else(|| {
                AppError::InternalServerError("Transcript not found for episode".into())
            })?;

        let chunks = generate_chunks(&transcript.data, 5.0)?;
        let processed = self.core.detect_ads(&chunks);
        let detection = Detection {
            episode_id: job.episode_id,
            segments: processed,
        };

        Ok(self.detection_repository.upsert(detection).await?)
    }
}
