use crate::error::AppError;
use crate::services::editor::core::Editor;
use crate::services::editor::types::{EditorJob, EditorResult};
use crate::services::editor::worker::EditorWorker;
use std::sync::Arc;
use tokio::sync::mpsc;

// todo pass in
const BEEP_BYTES: &[u8] = include_bytes!("../../assets/beep.mp3");

#[derive(Clone)]
pub struct EditorService {
    job_queue: mpsc::Sender<EditorJob>,
}

impl EditorService {
    pub fn new(
        core: Arc<dyn Editor>,
        callback: mpsc::Sender<EditorResult>,
        buffer: usize,
        concurrency_limit: usize,
    ) -> (Self, EditorWorker) {
        let (queue_send, queue_receive) = mpsc::channel::<EditorJob>(buffer);
        let service = Self {
            job_queue: queue_send,
        };
        let worker = EditorWorker {
            core,
            beep_audio: BEEP_BYTES,
            input_queue: queue_receive,
            callback,
            concurrency_limit,
        };
        (service, worker)
    }

    pub async fn edit_audio(&self, job: EditorJob) -> Result<(), AppError> {
        self.job_queue.send(job).await.map_err(|e| {
            tracing::error!("Edit queue rejected job: {}", e);
            AppError::InternalServerError("Edit queue is full or offline".into())
        })
    }
}
