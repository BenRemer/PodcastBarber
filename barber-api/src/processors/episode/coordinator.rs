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
                    self.handle_after_download(dl_result).await;
                }
                // Transcribe complete
                Some(tr_result) = self.transcribe_callback.recv() => {
                    tracing::info!("Transcription {} finished.", tr_result.episode_id);
                    if let Some(watcher) = &self.event_sender {
                        let _ = watcher.send(PipelineEvent::TranscriptionComplete(tr_result
                            .episode_id, tr_result.error.clone())).await;
                    }
                    self.handle_after_transcription(tr_result).await;
                }
                // Detection Complete
                Some(detection_result) = self.detection_callback.recv() => {
                    tracing::info!("Detection of {} finished.", detection_result.episode_id);
                    if let Some(watcher) = &self.event_sender {
                        let _ = watcher.send(PipelineEvent::DetectionComplete(detection_result
                            .episode_id, detection_result.error.clone())).await;
                    }
                    self.handle_after_detection(detection_result).await;
                }
                else => {
                    tracing::info!("All worker channels closed. Shutting down Coordinator.");
                    break;
                }
            }
        }
    }

    /// Set db to downloaded and start transcription
    async fn handle_after_download(&self, dl_result: DownloadResult) {
        let Ok(Some(mut episode)) = self.episode_repository.get(&dl_result.tracking_id).await
        else {
            tracing::info!("Episode {} not found.", dl_result.tracking_id);
            return;
        };

        let path = match dl_result.status {
            Ok(path) => path,
            Err(err) => {
                tracing::error!(
                    "Download failed for episode {}: {:?}",
                    dl_result.tracking_id,
                    err
                );
                episode.state = EpisodeState::Error;
                let _ = self.episode_repository.upsert(episode).await;
                return;
            }
        };

        tracing::info!("Download success for episode {}", dl_result.tracking_id);
        episode.state = EpisodeState::Downloaded;
        episode.local_file_path = Some(path.to_owned());
        let _ = self.episode_repository.upsert(episode).await;

        let _ = self
            .transcribe_service
            .transcribe_audio(TranscribeJob {
                episode_id: dl_result.tracking_id,
                file_path: path,
            })
            .await;
    }

    /// Set DB to transcribed and send to detection
    async fn handle_after_transcription(&self, tr_result: TranscribeResult) {
        let episode_id = tr_result.episode_id;

        if let Some(_) = tr_result.error {
            tracing::error!(
                "Transcription failed for {}: {:?}",
                episode_id,
                tr_result.error
            );
            return;
        };

        tracing::info!("Transcribe success for episode {}", episode_id);
        let mut episode = match self.episode_repository.get(&episode_id).await {
            Ok(Some(ep)) => ep,
            Ok(None) => {
                tracing::error!("Episode {} missing after transcription", episode_id);
                return;
            }
            Err(e) => {
                tracing::error!("Failed loading episode {}: {:?}", episode_id, e);
                return;
            }
        };

        episode.state = EpisodeState::Transcribed;
        if let Err(e) = self.episode_repository.upsert(episode).await {
            tracing::error!(
                "Failed updating episode {} after transcription: {:?}",
                episode_id,
                e
            );
            return;
        }

        let job = DetectionJob { episode_id };
        tracing::info!("Sending episode {} to detection queue", episode_id);

        if let Err(e) = self.detection_service.detect_ads(job).await {
            tracing::error!("Failed queueing detection {}: {:?}", episode_id, e);
        }
    }

    // Detection finished now... todo
    async fn handle_after_detection(&self, detection_result: DetectionResult) {
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
