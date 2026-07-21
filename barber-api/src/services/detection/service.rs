use crate::error::AppError;
use crate::services::detection::core::{DetectionConfig, DetectionCore};
use crate::services::detection::types::{DetectionJob, DetectionResult};
use crate::services::detection::worker::DetectionWorker;
use crate::storage::repository::detection::DetectionRepository;
use crate::storage::repository::transcript::TranscriptStore;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct DetectionService {
    job_queue: mpsc::Sender<DetectionJob>,
}

impl DetectionService {
    pub fn new(
        transcript_repository: Arc<dyn TranscriptStore>,
        detection_repository: DetectionRepository,
        callback: mpsc::Sender<DetectionResult>,
        buffer: usize,
    ) -> (Self, DetectionWorker) {
        let (queue_send, queue_receive) = mpsc::channel::<DetectionJob>(buffer);
        let core = DetectionCore::new(DetectionConfig::default());

        let service = Self {
            job_queue: queue_send,
        };
        let worker = DetectionWorker {
            transcript_repository,
            detection_repository,
            core,
            job_queue: queue_receive,
            callback,
        };
        (service, worker)
    }

    pub async fn detect_ads(&self, job: DetectionJob) -> Result<(), AppError> {
        self.job_queue.send(job).await.map_err(|e| {
            tracing::error!("Detection queue rejected job: {}", e);
            AppError::InternalServerError("Detection queue is full or offline".into())
        })
    }
}
