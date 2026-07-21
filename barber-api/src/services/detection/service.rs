use crate::error::AppError;
use crate::services::detection::chunker::DefaultChunker;
use crate::services::detection::core::{DetectionConfig, DetectionCore};
use crate::services::detection::types::{DetectionJob, DetectionResult};
use crate::services::detection::worker::DetectionWorker;
use crate::storage::repository::detection::DetectionStore;
use crate::storage::repository::transcript::TranscriptStore;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct DetectionService {
    job_queue: mpsc::Sender<DetectionJob>,
}

impl DetectionService {
    pub fn new(
        transcript_repository: Arc<dyn TranscriptStore>,
        detection_repository: Arc<dyn DetectionStore>,
        callback: mpsc::Sender<DetectionResult>,
        buffer: usize,
        concurrency_limit: usize,
    ) -> (Self, DetectionWorker) {
        let (queue_send, queue_receive) = mpsc::channel::<DetectionJob>(buffer);
        // todo pass in
        let core = Arc::new(DetectionCore::new(DetectionConfig::default()));
        let chunker = Arc::new(DefaultChunker);

        let service = Self {
            job_queue: queue_send,
        };
        let worker = DetectionWorker {
            transcript_repository,
            detection_repository,
            core,
            chunker,
            concurrency_limit,
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
