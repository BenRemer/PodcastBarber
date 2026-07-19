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
            match self
                .core
                .transcribe_audio(job.file_name, job.content_type, job.data)
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
                        Ok(transcript) => {
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

    async fn send_result(
        &self,
        episode_id: Uuid,
        error: Option<AppError>,
    ) {
        let _ = self
            .callback
            .send(TranscribeResult {
                episode_id,
                error,
            })
            .await;
    }
}
