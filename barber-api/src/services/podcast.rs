use std::path::PathBuf;
use std::sync::Arc;
use reqwest::Client;
use crate::error::AppError;
use crate::models::podcast::Podcast;
use crate::services::rss::RSSFeedService;
use crate::storage::manager::DownloadManager;
use crate::storage::repository::podcast::PodcastRepository;

#[derive(Clone)]
pub struct PodcastService {
    client: Client,
    podcast_repository: PodcastRepository,
    rss_service: RSSFeedService,
    download_manager: Arc<DownloadManager>
}

impl PodcastService {
    pub fn new(podcast_repository: PodcastRepository, rss_service: RSSFeedService,
               download_manager: Arc<DownloadManager>
    ) -> Self {
        Self {
            client: Client::new(),
            podcast_repository,
            rss_service,
            download_manager
        }
    }

    pub async fn list_podcasts(&self) -> Result<Vec<Podcast>, AppError> {
        self.podcast_repository.get_all().await
    }

    pub async fn subscribe_podcast(
        &self, feed_url: &str
    ) -> Result<Podcast, AppError> {
        let metadata = self.rss_service.fetch_podcast_metadata(feed_url).await?;
        let podcast: Podcast = metadata.into();
        self.podcast_repository.insert(podcast).await
    }

    pub async fn is_subscribed(&self, feed_url: &str) -> Result<bool, AppError> {
        self.podcast_repository.is_subscribed_feed(feed_url).await
    }

    // todo move to episode
    pub async fn download_episode(
        &self, feed_url: &str, guid: &str
    ) -> Result<PathBuf, AppError> {
        let channel = self.rss_service.construct_rss_channel(feed_url).await?;

        let podcast_name = channel.title().to_string();
        let metadata = channel.items()
            .iter()
            .find_map(|item| {
                match RSSFeedService::extract_metadata(item) {
                    Ok(meta) if meta.guid == guid => Some(meta),
                    _ => None,
                }
            })
            .ok_or_else(|| {
                tracing::warn!("Episode with ID {} not found in feed", guid);
                AppError::NotFound
            })?;

        self.download_manager
            .download_to_path(&self.client, &metadata.audio_url, &podcast_name, &metadata.guid)
            .await
            .map_err(|_| AppError::InternalServerError("Failed to download audio".into()))
    }


    // Fetches the feed, finds the newest episode, and streams the MP3 to disk
    pub async fn download_latest_episode(
        &self, feed_url: &str
    ) -> Result<PathBuf, AppError> {
        tracing::info!("Fetching RSS feed from: {}", feed_url);

        let channel =  self.rss_service.construct_rss_channel(feed_url).await?;
        let podcast_name = channel.title().to_string();

        let latest_item = channel.items().first()
            .ok_or_else(|| AppError::InternalServerError("No episodes found in feed".into()))?;

        let metadata =  RSSFeedService::extract_metadata(latest_item).map_err(|e| {
            tracing::warn!("No episodes found in feed: {:?}", e);
            AppError::NotFound
        })?;

        let file_path = self.download_manager
            .download_to_path(&self.client, &metadata.audio_url, &podcast_name, &metadata.guid)
            .await
            .map_err(|_| AppError::InternalServerError("Failed to download audio".into()))?;

        Ok(file_path)
    }
}
