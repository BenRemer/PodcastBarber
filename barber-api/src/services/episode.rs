use reqwest::Client;
use crate::error::AppError;
use crate::models::episode::{Episode, EpisodeState};
use crate::storage::manager::DownloadManager;
use crate::storage::repository::episode::EpisodeRepository;

#[derive(Clone)]
pub struct EpisodeService {
    episode_repository: EpisodeRepository,
    download_manager: DownloadManager,
    client: Client,
}

impl EpisodeService {
    pub fn new(repo: EpisodeRepository, manager: DownloadManager) -> Self {
        Self {
            episode_repository: repo,
            download_manager: manager,
            client: Client::new(),
        }
    }

    // pub async fn queue_episode_download(
    //     &self, feed_url: &str, guid: &str
    // ) -> Result<Episode, AppError> {
    //     // 1. Check and Subscribe (Idempotent: updates if already exists, inserts if not)
    //     let podcast = self.subscribe_podcast(feed_url).await?;
    //
    //     // Fetch RSS to get the target episode metadata
    //     let channel = self.rss_service.construct_rss_channel(feed_url).await?;
    //     let metadata = channel.items()
    //         .iter()
    //         .find_map(|item| {
    //             match RSSFeedService::extract_metadata(item) {
    //                 Ok(meta) if meta.guid == guid => Some(meta),
    //                 _ => None,
    //             }
    //         })
    //         .ok_or_else(|| {
    //             tracing::warn!("Episode with ID {} not found in feed", guid);
    //             AppError::NotFound
    //         })?;
    //
    //     // 2. Create the Episode and save it as Pending
    //     let pending_episode = Episode {
    //         id: Uuid::new_v4(), // Generate a unique ID for this download
    //         podcast_id: podcast.id,
    //         guid: metadata.guid.clone(),
    //         title: metadata.title.clone(),
    //         audio_url: metadata.audio_url.clone(),
    //         local_file_path: None,
    //         state: EpisodeState::Pending,
    //     };
    //
    //     // Insert into the database
    //     let saved_episode = self.episode_repository.upsert(pending_episode).await?;
    //
    //     // 3. Queue the download in a background task
    //     // We clone the necessary components. reqwest::Client, SqlitePool (inside the repo),
    //     // and Arc-wrapped managers are all very cheap to clone in Rust.
    //     let download_manager = self.download_manager.clone();
    //     let episode_repo = self.episode_repository.clone();
    //     let client = self.client.clone();
    //     let podcast_title = podcast.title.clone();
    //
    //     let mut processing_episode = saved_episode.clone();
    //
    //     tokio::spawn(async move {
    //         // Optional: Mark as actively processing right before download starts
    //         processing_episode.state = EpisodeState::Processing;
    //         let _ = episode_repo.upsert(processing_episode.clone()).await;
    //
    //         match download_manager
    //             .download_to_path(&client, &processing_episode.audio_url, &podcast_title, &processing_episode.guid)
    //             .await
    //         {
    //             Ok(path) => {
    //                 tracing::info!("Successfully downloaded: {}", processing_episode.title);
    //                 processing_episode.state = EpisodeState::Downloaded;
    //                 processing_episode.local_file_path = Some(path.to_string_lossy().into_owned());
    //
    //                 // Final database update to mark completion
    //                 let _ = episode_repo.upsert(processing_episode).await;
    //             }
    //             Err(e) => {
    //                 tracing::error!("Failed to download {}: {:?}", processing_episode.title, e);
    //                 processing_episode.state = EpisodeState::Error;
    //
    //                 // Update database to reflect the failure
    //                 let _ = episode_repo.upsert(processing_episode).await;
    //             }
    //         }
    //     });
    //
    //     // Immediately return the Pending episode to Axum so the UI can show a loading spinner
    //     Ok(saved_episode)
    // }
}