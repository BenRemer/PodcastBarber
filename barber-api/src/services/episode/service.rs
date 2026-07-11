use crate::error::AppError;
use crate::models::episode::Episode;
use crate::models::podcast::Podcast;
use crate::services::episode::worker::EpisodeWorker;
use crate::storage::download::{DownloadJob, DownloadManager, DownloadResult};
use crate::storage::repository::episode::EpisodeRepository;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Clone)]
pub struct EpisodeService {
    episode_repository: EpisodeRepository,
    download_manager: Arc<DownloadManager>,
}

impl EpisodeService {
    pub fn new(
        episode_repository: EpisodeRepository,
        download_manager: Arc<DownloadManager>,
        download_callback: mpsc::Receiver<DownloadResult>,
    ) -> (Self, EpisodeWorker) {
        let worker = EpisodeWorker {
            repo: episode_repository.clone(),
            download_callback,
        };

        let service = Self {
            episode_repository,
            download_manager,
        };

        (service, worker)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<Episode>, AppError> {
        self.episode_repository.get(id).await
    }

    pub async fn get_episodes_by_podcast(
        &self,
        podcast_id: &Uuid,
    ) -> Result<Vec<Episode>, AppError> {
        self.episode_repository.get_by_podcast_id(podcast_id).await
    }

    pub async fn queue_episode_download(
        &self,
        podcast: Podcast,
        episode: Episode,
    ) -> Result<Episode, AppError> {
        // Insert into the database
        let saved_episode = self.episode_repository.upsert(episode).await?;

        // Drop the job into the Manager's queue.
        let job = DownloadJob {
            uuid: saved_episode.id.clone(),
            audio_url: saved_episode.audio_url.clone(),
            folder_name: podcast.id.to_string(),
            guid: saved_episode.guid.clone(),
        };
        self.download_manager.enqueue_download(job).await?;

        // Return pending episode
        Ok(saved_episode)
    }

    pub async fn delete_episode(&self, episode: Episode) -> Result<(), AppError> {
        let found = self.episode_repository.delete(&episode.id).await?;

        if !found {
            return Err(AppError::NotFound);
        };

        if let Some(path) = &episode.local_file_path {
            let _ = tokio::fs::remove_file(&path).await;
        };

        Ok(())
    }
}
