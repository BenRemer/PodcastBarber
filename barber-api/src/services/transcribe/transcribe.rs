use crate::error::AppError;
use crate::services::transcribe::core::Transcriber;
use crate::services::transcribe::types::{TranscribeJob, TranscribeResult};
use crate::services::transcribe::worker::TranscribeWorker;
use crate::storage::repository::transcript::TranscriptStore;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct TranscribeService {
    job_queue: mpsc::Sender<TranscribeJob>,
    core: Arc<dyn Transcriber>,
}

impl TranscribeService {
    pub fn new(
        core: Arc<dyn Transcriber>,
        callback: mpsc::Sender<TranscribeResult>,
        buffer: usize,
        concurrency_limit: usize,
        transcript_repository: Arc<dyn TranscriptStore>,
    ) -> (Self, TranscribeWorker) {
        let (queue_send, queue_receive) = mpsc::channel::<TranscribeJob>(buffer);
        let service = Self {
            job_queue: queue_send,
            core: core.clone(),
        };
        let worker = TranscribeWorker {
            transcript_repository,
            core,
            queue_receive,
            callback,
            concurrency_limit,
        };
        (service, worker)
    }

    pub async fn check_health(&self) -> Result<Value, AppError> {
        self.core.check_health().await
    }

    pub async fn transcribe_audio(&self, job: TranscribeJob) -> Result<(), AppError> {
        self.job_queue.send(job).await.map_err(|e| {
            tracing::error!("Translate queue rejected job: {}", e);
            AppError::InternalServerError("Translate queue is full or offline".into())
        })
    }
}
