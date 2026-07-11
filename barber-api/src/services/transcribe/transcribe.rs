use crate::error::AppError;
use crate::services::transcribe::core::TranscribeCore;
use crate::services::transcribe::types::{TranscribeJob, TranscribeResult};
use crate::services::transcribe::worker::TranscribeWorker;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct TranscribeService {
    job_queue: mpsc::Sender<TranscribeJob>,
    core: TranscribeCore,
}

impl TranscribeService {
    pub fn new(
        base_url: String,
        client: Client,
        callback: mpsc::Sender<TranscribeResult>,
        buffer: usize,
    ) -> (Self, TranscribeWorker) {
        let (queue_send, queue_receive) = mpsc::channel::<TranscribeJob>(buffer);
        let core = TranscribeCore::new(base_url, client);
        let service = Self {
            job_queue: queue_send,
            core: core.clone(),
        };
        let worker = TranscribeWorker {
            core,
            queue_receive,
            callback,
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
