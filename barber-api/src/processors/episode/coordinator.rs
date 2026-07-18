use crate::error::AppError;
use crate::models::episode::EpisodeState;
use crate::services::detection::{DetectionJob, DetectionResult, DetectionService};
use crate::services::transcribe::TranscribeService;
use crate::services::transcribe::types::{TranscribeJob, TranscribeResult};
use crate::storage::download::DownloadResult;
use crate::storage::repository::episode::EpisodeRepository;
use crate::storage::repository::transcript::TranscriptRepository;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum PipelineEvent {
    DownloadComplete(Uuid),
    TranscriptionComplete(Uuid, Option<AppError>),
    DetectionComplete(Uuid, Option<AppError>),
}

pub struct AudioCoordinator {
    pub download_callback: mpsc::Receiver<DownloadResult>,
    pub transcribe_service: Arc<TranscribeService>,
    pub transcribe_callback: mpsc::Receiver<TranscribeResult>,
    pub detection_service: Arc<DetectionService>,
    pub detection_callback: mpsc::Receiver<DetectionResult>,
    pub episode_repository: EpisodeRepository,
    pub transcript_repository: TranscriptRepository,
    pub event_sender: Option<mpsc::Sender<PipelineEvent>>,
}

// todo background cleanup tasks
impl AudioCoordinator {
    pub fn new(
        download_callback: mpsc::Receiver<DownloadResult>,
        transcribe_service: Arc<TranscribeService>,
        transcribe_callback: mpsc::Receiver<TranscribeResult>,
        detection_service: Arc<DetectionService>,
        detection_callback: mpsc::Receiver<DetectionResult>,
        episode_repository: EpisodeRepository,
        transcript_repository: TranscriptRepository,
        event_sender: Option<mpsc::Sender<PipelineEvent>>,
    ) -> Self {
        Self {
            download_callback,
            transcribe_service,
            transcribe_callback,
            detection_service,
            detection_callback,
            episode_repository,
            transcript_repository,
            event_sender,
        }
    }

    /// Gets results and passes to handler
    pub async fn run(mut self) {
        tracing::info!("Starting Audio Coordinator Processor...");

        loop {
            tokio::select! {
                // Episode Download Finished
                Some(dl_result) = self.download_callback.recv() => {
                    tracing::info!("Download of episode {} finished, \
                        sending to get transcribed.", dl_result.tracking_id);
                    if let Some(watcher) = &self.event_sender {
                        let _ = watcher.send(PipelineEvent::DownloadComplete(dl_result.tracking_id)).await;
                    }
                    self.handle_download(dl_result).await;
                }
                // Transcribe complete
                Some(tr_result) = self.transcribe_callback.recv() => {
                    tracing::info!("Transcription {} finished.", tr_result.episode_id);
                    if let Some(watcher) = &self.event_sender {
                        let _ = watcher.send(PipelineEvent::TranscriptionComplete(tr_result
                            .episode_id, tr_result.error.clone())).await;
                    }
                    self.handle_transcription(tr_result).await;
                }
                // Detection Complete
                Some(detection_result) = self.detection_callback.recv() => {
                    tracing::info!("Detection of {} finished.", detection_result.episode_id);
                    if let Some(watcher) = &self.event_sender {
                        let _ = watcher.send(PipelineEvent::DetectionComplete(detection_result
                            .episode_id, detection_result.error.clone())).await;
                    }
                    self.handle_detection(detection_result).await;
                }
                else => {
                    tracing::info!("All worker channels closed. Shutting down Coordinator.");
                    break;
                }
            }
        }
    }

    /// Set db to downloaded and start transcription
    async fn handle_download(&self, dl_result: DownloadResult) {
        if let Ok(Some(mut episode)) = self.episode_repository.get(&dl_result.tracking_id).await {
            match dl_result.status {
                Ok(path) => {
                    tracing::info!("Download success for episode {}", dl_result.tracking_id);

                    episode.state = EpisodeState::Downloaded;
                    episode.local_file_path = Some(path.to_string_lossy().into_owned());
                    let _ = self.episode_repository.upsert(episode).await;

                    // Read the file into bytes and send it to the Transcribe worker
                    if let Ok(file_bytes) = tokio::fs::read(&path).await {
                        let content_type = match infer::get(&file_bytes) {
                            Some(kind) => kind.mime_type().to_string(),
                            None => "application/octet-stream".to_string(),
                        };

                        let transcribe_job = TranscribeJob {
                            episode_id: dl_result.tracking_id,
                            file_name: path.file_name().unwrap().to_string_lossy().into_owned(),
                            content_type,
                            data: file_bytes.into(),
                        };

                        let _ = self
                            .transcribe_service
                            .transcribe_audio(transcribe_job)
                            .await;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Download failed for episode {}: {:?}",
                        dl_result.tracking_id,
                        e
                    );
                    episode.state = EpisodeState::Error;
                    let _ = self.episode_repository.upsert(episode).await;
                }
            }
        }
    }

    /// Set DB to transcribed and send to detection
    async fn handle_transcription(&self, tr_result: TranscribeResult) {
        match tr_result.transcription {
            Some(_) => {
                tracing::info!("Transcribe success for episode {}", tr_result.episode_id);
                match self.episode_repository.get(&tr_result.episode_id).await {
                    Ok(Some(mut episode)) => {
                        episode.state = EpisodeState::Transcribed;
                        if let Err(e) = self.episode_repository.upsert(episode).await {
                            tracing::error!(
                                "Failed updating episode {} after transcription: {:?}",
                                tr_result.episode_id,
                                e
                            );
                            return;
                        }

                        let job = DetectionJob {
                            episode_id: tr_result.episode_id,
                        };
                        tracing::info!(
                            "Sending episode {} to detection queue",
                            tr_result.episode_id
                        );
                        if let Err(e) = self.detection_service.detect_ads(job).await {
                            tracing::error!(
                                "Failed queueing detection {}: {:?}",
                                tr_result.episode_id,
                                e
                            );
                        }
                    }
                    Ok(None) => {
                        tracing::error!(
                            "Episode {} missing after transcription",
                            tr_result.episode_id
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed loading episode {}: {:?}", tr_result.episode_id, e);
                    }
                }
            }
            None => {
                tracing::error!(
                    "Transcription failed for {}: {:?}",
                    tr_result.episode_id,
                    tr_result.error
                );
            }
        }
    }

    // Detection finished now... todo
    async fn handle_detection(&self, detection_result: DetectionResult) {
        if let Ok(Some(mut _episode)) = self
            .transcript_repository // todo needs to be detection repo
            .get_by_episode_id(&detection_result.episode_id)
            .await
        {
            println!("Detected episode {}", detection_result.episode_id);
        } else {
            tracing::error!("Detection missing episode {}", detection_result.episode_id);
        }
    }
}
