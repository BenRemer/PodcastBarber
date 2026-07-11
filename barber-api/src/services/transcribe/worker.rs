use crate::services::transcribe::core::TranscribeCore;
use crate::services::transcribe::types::{TranscribeJob, TranscribeResult};
use tokio::sync::mpsc;

pub struct TranscribeWorker {
    pub(crate) core: TranscribeCore,
    pub(crate) queue_receive: mpsc::Receiver<TranscribeJob>,
    pub(crate) callback: mpsc::Sender<TranscribeResult>,
}

impl TranscribeWorker {
    pub async fn run(mut self) {
        tracing::info!("Starting Transcribe Worker...");
        while let Some(job) = self.queue_receive.recv().await {
            match self
                .core
                .transcribe_audio(job.file_name, job.content_type, job.data)
                .await
            {
                Ok(transcript) => {
                    tracing::info!("Transcription successful!");
                    let _ = self
                        .callback
                        .send(TranscribeResult {
                            transcription: Some(transcript),
                            error: None,
                        })
                        .await;
                }
                Err(e) => {
                    tracing::error!("Transcription job failed: {:?}", e);
                    let _ = self
                        .callback
                        .send(TranscribeResult {
                            transcription: None,
                            error: Some(e),
                        })
                        .await;
                }
            }
        }
        tracing::info!("Stopping Transcribe Worker.");
    }
}
