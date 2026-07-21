use crate::error::AppError;
use crate::models::transcript::Transcript;
use crate::services::transcribe::core::TranscribeCore;
use crate::services::transcribe::types::{TranscribeJob, TranscribeResult};
use crate::storage::repository::transcript::TranscriptRepository;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct TranscribeWorker {
    pub transcript_repository: TranscriptRepository,
    pub(crate) core: TranscribeCore,
    pub(crate) queue_receive: mpsc::Receiver<TranscribeJob>,
    pub(crate) callback: mpsc::Sender<TranscribeResult>,
}

impl TranscribeWorker {
    pub async fn run(mut self) {
        tracing::info!("Starting Transcribe Worker...");
        while let Some(job) = self.queue_receive.recv().await {
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

            match self
                .core
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
                    match self.transcript_repository.upsert(transcript).await {
                        Ok(_) => {
                            self.send_result(episode_id, None).await;
                        }
                        Err(e) => {
                            self.send_result(episode_id, Some(e)).await;
                        }
                    }
                }
                Err(e) => {
                    self.send_result(episode_id, Some(e)).await;
                }
            }
        }
        tracing::info!("Stopping Transcribe Worker.");
    }

    async fn send_result(&self, episode_id: Uuid, error: Option<AppError>) {
        let _ = self
            .callback
            .send(TranscribeResult { episode_id, error })
            .await;
    }
}
