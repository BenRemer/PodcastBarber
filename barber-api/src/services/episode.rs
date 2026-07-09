use std::sync::Arc;
use uuid::Uuid;
use crate::error::AppError;
use crate::models::episode::{Episode, EpisodeState};
use crate::models::podcast::Podcast;
use crate::storage::manager::DownloadManager;
use crate::storage::repository::episode::EpisodeRepository;

#[derive(Clone)]
pub struct EpisodeService {
    episode_repository: EpisodeRepository,
    download_manager: Arc<DownloadManager>,
}

impl EpisodeService {
    pub fn new(repo: EpisodeRepository, manager: Arc<DownloadManager>) -> Self {
        Self {
            episode_repository: repo,
            download_manager: manager,
        }
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Episode>, AppError> {
        self.episode_repository.get(id).await
    }

    pub async fn get_episodes_by_podcast(
        &self,
        podcast_id: &Uuid
    ) -> Result<Vec<Episode>, AppError> {
        self.episode_repository.get_by_podcast_id(podcast_id).await
    }

    pub async fn queue_episode_download(
        &self, podcast: Podcast, episode: Episode
    ) -> Result<Episode, AppError> {
        // Insert into the database
        let saved_episode = self.episode_repository.upsert(episode).await?;

        let download_manager = self.download_manager.clone();
        let episode_repo = self.episode_repository.clone();
        let podcast_title = podcast.title.clone();

        let mut processing_episode = saved_episode.clone();

        // spawn task to download async
        tokio::spawn(async move {
            // processing_episode.state = EpisodeState::Processing;
            if let Err(e) = episode_repo.upsert(processing_episode.clone()).await {
                tracing::error!("Failed to update processing state: {:?}", e);
            }

            match download_manager
                .download_to_path(&processing_episode.audio_url, &podcast_title, &processing_episode.guid)
                .await
            {
                Ok(path) => {
                    tracing::info!("Successfully downloaded: {}", processing_episode.title);
                    processing_episode.state = EpisodeState::Downloaded;
                    processing_episode.local_file_path = Some(path.to_string_lossy().into_owned());

                    // Final database update to mark completion
                    match episode_repo.upsert(processing_episode).await {
                        Ok(_) => tracing::info!("Episode updated successfully"),
                        Err(e) => tracing::error!("Failed to update episode: {:?}", e),
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to download {}: {:?}", processing_episode.title, e);
                    processing_episode.state = EpisodeState::Error;

                    // Update database to reflect the failure
                    match episode_repo.upsert(processing_episode).await {
                        Ok(_) => tracing::info!("Episode updated successfully"),
                        Err(e) => tracing::error!("Failed to update episode: {:?}", e),
                    }
                }
            }
        });

        Ok(saved_episode)
    }
}