use crate::models::episode::EpisodeState;
use crate::services::transcribe::TranscribeService;
use crate::services::transcribe::types::{TranscribeJob, TranscribeResult};
use crate::storage::download::DownloadResult;
use crate::storage::repository::episode::EpisodeRepository;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AudioCoordinator {
    pub download_callback: mpsc::Receiver<DownloadResult>,
    pub transcribe_service: Arc<TranscribeService>,
    pub transcribe_callback: mpsc::Receiver<TranscribeResult>,
    pub repo: EpisodeRepository,
}

impl AudioCoordinator {
    pub fn new(
        download_callback: mpsc::Receiver<DownloadResult>,
        transcribe_service: Arc<TranscribeService>,
        transcribe_callback: mpsc::Receiver<TranscribeResult>,
        repo: EpisodeRepository,
    ) -> Self {
        Self {
            download_callback,
            transcribe_service,
            transcribe_callback,
            repo,
        }
    }
    /// Sees download of an episode finished, mark episode as downloaded, start transcribing
    pub async fn run(mut self) {
        tracing::info!("Starting Audio Coordinator Processor...");

        loop {
            tokio::select! {
                // Episode Download Finished
                Some(dl_result) = self.download_callback.recv() => {
                    if let Ok(Some(mut episode)) = self.repo.get(&dl_result.tracking_id).await {
                        match dl_result.status {
                            Ok(path) => {
                                tracing::info!("Download success for episode {}", dl_result.tracking_id);

                                episode.state = EpisodeState::Downloaded;
                                episode.local_file_path = Some(path.to_string_lossy().into_owned());
                                let _ = self.repo.upsert(episode).await;

                                // Read the file into bytes and send it to the Transcribe worker
                                if let Ok(file_bytes) = tokio::fs::read(&path).await {
                                    let content_type = match infer::get(&file_bytes) {
                                        Some(kind) => kind.mime_type().to_string(),
                                        None => "application/octet-stream".to_string(),
                                    };

                                    let transcribe_job = TranscribeJob {
                                        tracking_id: dl_result.tracking_id,
                                        file_name: path.file_name().unwrap().to_string_lossy().into_owned(),
                                        content_type,
                                        data: file_bytes.into(),
                                    };

                                    // Send to get transcribed
                                    let _ = self.transcribe_service.transcribe_audio(transcribe_job).await;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Download failed for episode {}: {:?}", dl_result.tracking_id, e);
                                episode.state = EpisodeState::Error;
                                let _ = self.repo.upsert(episode).await;
                            }
                        }
                    }
                }
                // Transcribe complete
                Some(tr_result) = self.transcribe_callback.recv() => {
                    tracing::info!("Caught transcription for {}", tr_result.tracking_id);
                    if let Ok(Some(episode)) = self.repo.get(&tr_result.tracking_id).await {
                        match tr_result.transcription {
                            Some(transcription) => {
                                tracing::info!("Transcribe success for episode {}", tr_result.tracking_id);
                                let _ = self.repo.update_transcript(&tr_result.tracking_id,
                                    &transcription.to_string()).await;
                            }
                            None => {
                                tracing::error!("Caught transcription for {}: {:?}", tr_result
                                    .tracking_id, tr_result.error);
                            }
                        }
                    } else {
                        tracing::error!("Episode missing for {}", tr_result.tracking_id);
                    }
                }
                else => {
                    tracing::info!("All worker channels closed. Shutting down Coordinator.");
                    break;
                }
            }
        }
    }
}
