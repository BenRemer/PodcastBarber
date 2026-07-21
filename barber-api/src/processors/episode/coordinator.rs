use crate::error::AppError;
use crate::models::episode::{Episode, EpisodeState};
use crate::services::detection::{DetectionJob, DetectionResult, DetectionService};
use crate::services::editor::{EditorJob, EditorResult, EditorService};
use crate::services::transcribe::TranscribeService;
use crate::services::transcribe::types::{TranscribeJob, TranscribeResult};
use crate::storage::download::DownloadResult;
use crate::storage::repository::detection::DetectionStore;
use crate::storage::repository::episode::EpisodeRepository;
use crate::storage::repository::transcript::TranscriptStore;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum PipelineEvent {
    DownloadComplete(Uuid),
    TranscriptionComplete(Uuid, Option<AppError>),
    DetectionComplete(Uuid, Option<AppError>),
    EditComplete(Uuid),
}

pub struct AudioCoordinator {
    pub download_callback: mpsc::Receiver<DownloadResult>,
    pub episode_repository: EpisodeRepository,

    pub transcribe_service: Arc<TranscribeService>,
    pub transcribe_callback: mpsc::Receiver<TranscribeResult>,
    pub transcript_repository: Arc<dyn TranscriptStore>,

    pub detection_service: Arc<DetectionService>,
    pub detection_callback: mpsc::Receiver<DetectionResult>,
    pub detection_repository: Arc<dyn DetectionStore>,

    pub editor_service: Arc<EditorService>,
    pub editor_callback: mpsc::Receiver<EditorResult>,

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
        editor_service: Arc<EditorService>,
        editor_callback: mpsc::Receiver<EditorResult>,
        episode_repository: EpisodeRepository,
        transcript_repository: Arc<dyn TranscriptStore>,
        detection_repository: Arc<dyn DetectionStore>,
        event_sender: Option<mpsc::Sender<PipelineEvent>>,
    ) -> Self {
        Self {
            download_callback,
            transcribe_service,
            transcribe_callback,
            detection_service,
            detection_callback,
            editor_service,
            editor_callback,
            episode_repository,
            transcript_repository,
            detection_repository,
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
                    let id = dl_result.tracking_id;
                    self.handle_after_download(dl_result).await;
                    if let Some(watcher) = &self.event_sender {
                        let _ = watcher.send(PipelineEvent::DownloadComplete(id)).await;
                    }
                }
                // Transcribe complete
                Some(tr_result) = self.transcribe_callback.recv() => {
                    tracing::info!("Transcription {} finished.", tr_result.episode_id);
                    self.handle_after_transcription(&tr_result).await;
                    if let Some(watcher) = &self.event_sender {
                        let _ = watcher.send(PipelineEvent::TranscriptionComplete(tr_result
                            .episode_id, tr_result.error)).await;
                    }
                }
                // Detection Complete
                Some(detection_result) = self.detection_callback.recv() => {
                    tracing::info!("Detection of {} finished.", detection_result.episode_id);
                    self.handle_after_detection(&detection_result).await;
                    if let Some(watcher) = &self.event_sender {
                        let _ = watcher.send(PipelineEvent::DetectionComplete(detection_result
                            .episode_id, detection_result.error.clone())).await;
                    }
                }
                Some(editor_result) = self.editor_callback.recv() => {
                    tracing::info!("Edit of episode finished");
                    self.handle_after_edit(&editor_result).await;
                    if let Some(watcher) = &self.event_sender {
                        let _ = watcher.send(PipelineEvent::EditComplete(editor_result.episode_id())).await;
                    }
                }
                else => {
                    tracing::info!("All worker channels closed. Shutting down Coordinator.");
                    break;
                }
            }
        }
    }

    async fn update_episode_state(
        &self,
        episode_id: &Uuid,
        state: EpisodeState,
    ) -> Option<Episode> {
        let Ok(Some(mut episode)) = self.episode_repository.get(episode_id).await else {
            tracing::info!("Episode {} not found.", episode_id);
            return None;
        };

        episode.state = state;
        if let Err(e) = self.episode_repository.upsert(episode.clone()).await {
            tracing::error!(
                "Failed updating episode {} after detection: {:?}",
                episode_id,
                e
            );
            return None;
        }

        Some(episode)
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
    async fn handle_after_transcription(&self, tr_result: &TranscribeResult) {
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
        if self
            .update_episode_state(&episode_id, EpisodeState::Transcribed)
            .await
            .is_none()
        {
            tracing::error!(
                "Failed to update episode {} for transcribed tracking",
                episode_id
            );
            return;
        }

        let job = DetectionJob { episode_id };
        tracing::info!("Sending episode {} to detection queue", episode_id);

        if let Err(e) = self.detection_service.detect_ads(job).await {
            tracing::error!("Failed queueing detection {}: {:?}", episode_id, e);
        }
    }

    // Detection finished now send to editor
    async fn handle_after_detection(&self, detection_result: &DetectionResult) {
        let episode_id = &detection_result.episode_id;
        if detection_result.error.is_some() {
            tracing::error!("Detection failed for episode {}", episode_id);
            return;
        }

        let Some(episode) = self
            .update_episode_state(episode_id, EpisodeState::Detected)
            .await
        else {
            tracing::error!(
                "Failed to update episode {} for detected tracking",
                episode_id
            );
            return;
        };

        let Ok(Some(detection)) = self
            .detection_repository
            .get_detection_by_episode(episode_id)
            .await
        else {
            tracing::info!("Detection {} not found.", episode_id);
            return;
        };

        if let Err(e) = self
            .editor_service
            .edit_audio(EditorJob {
                episode_id: detection.episode_id,
                episode_path: episode.local_file_path.unwrap(),
                segments: detection.segments,
            })
            .await
        {
            tracing::error!(
                "Failed sending episode to edit queue {}: {:?}",
                detection.episode_id,
                e
            );
        };
    }

    // after edit todo
    async fn handle_after_edit(&self, editor_result: &EditorResult) {
        let EditorResult::Success { episode_id, path } = editor_result else {
            if let EditorResult::Failure { episode_id, error } = editor_result {
                tracing::error!("Failed to process episode {}: {:?}", episode_id, error);
            }
            return;
        };

        tracing::info!("Successfully processed to {:?}", path);
        if self
            .update_episode_state(&episode_id, EpisodeState::Edited)
            .await
            .is_none()
        {
            tracing::error!(
                "Failed to update episode {} for edited tracking",
                episode_id
            );
            return;
        }
        // todo
    }
}
