use reqwest::Client;
use rss::{Channel, Item};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::error::AppError;
use crate::storage::download_manager::DownloadManager;

#[derive(Clone)]
pub struct RSSFeedService {
    client: Client,
    download_manager: Arc<DownloadManager>
}

impl RSSFeedService {
    pub fn new(download_manager: Arc<DownloadManager>) -> Self {
        Self {
            client: Client::new(),
            download_manager
        }
    }

    async fn construct_rss_channel(&self, feed_url: &str) -> Result<Channel, AppError> {
        let xml_bytes = self.client.get(feed_url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch RSS: {}", e);
                AppError::InternalServerError("Failed to reach feed URL".into())
            })?
            .bytes()
            .await
            .map_err(|_| AppError::InternalServerError("Failed to read XML payload".into()))?;

        let channel = Channel::read_from(&xml_bytes[..])
            .map_err(|e| {
                println!("!!! RSS PARSING CRASHED BECAUSE: {:#?} !!!", e);
                tracing::error!("Failed to parse RSS: {}", e);
                AppError::InternalServerError("Invalid RSS format".into())
            })?;

        Ok(channel)
    }

    pub async fn _download_episode(
        &self,
        audio_url: &str,
        title: &str,
        guid: &str,
    ) -> Result<PathBuf, AppError> {
        // Get the prepared path from the manager
        let output_file = self.download_manager
            .prepare_episode_path(title, guid)
            .await
            .map_err(|e| {
                tracing::error!("Failed to prepare download path: {}", e);
                AppError::InternalServerError("Failed to prepare storage".into())
            })?;

        tracing::info!("Streaming audio to: {}", output_file.display());

        // Open the HTTP stream
        let mut response = self.client.get(audio_url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to start download: {}", e);
                AppError::InternalServerError("Audio download failed".into())
            })?;

        // Open a file on the local disk
        let mut file = File::create(&output_file).await
            .map_err(|e| {
                tracing::error!("Failed to create local file at {:?}: {}", output_file, e);
                AppError::InternalServerError("Failed to create local file".into())
            })?;

        // Stream the file directly to disk chunk-by-chunk
        while let Some(chunk) = response.chunk().await.map_err(|_| {
            AppError::InternalServerError("Interrupted while streaming audio".into())
        })? {
            file.write_all(&chunk).await
                .map_err(|_| AppError::InternalServerError("Failed to write chunk to disk".into()))?;
        }

        tracing::info!("Successfully saved to {}", output_file.display());

        Ok(output_file)
    }

    pub async fn list_episodes(
        &self, feed_url: &str, limit: usize
    ) -> Result<Vec<Item>, AppError> {
        tracing::info!("Listing {} episodes of {}", limit, feed_url);

        let channel = self.construct_rss_channel(feed_url).await?;

        let items: Vec<Item> = channel.items
            .iter()
            .take(limit)
            .cloned()
            .collect();

        Ok(items)
    }

    pub async fn download_episode(
        &self, feed_url: &str, id: &str, output_dir: &str
    ) -> Result<PathBuf, AppError> {
        tracing::info!("Downloading {} episodes of {}", id, feed_url);

        let channel = self.construct_rss_channel(feed_url).await?;

        let episode = channel.items()
            .into_iter()
            .find(|item| {
                item.guid().is_some_and(|guid| guid.value() == id)
            })
            .ok_or_else(|| {
                tracing::warn!("Episode with GUID {} not found", id);
                AppError::NotFound
            })?;

        let podcast_name = episode.title().expect("No Title");
        let guid = episode.guid().expect("No GUID").value().to_string();
        let enclosure = episode.enclosure().ok_or_else(|| AppError::NotFound)?;

        self.download_manager
            .download_to_path(&self.client, &enclosure.url, &podcast_name, &guid)
            .await
    }

    // Fetches the feed, finds the newest episode, and streams the MP3 to disk
    pub async fn download_latest_episode(
        &self, feed_url: &str, output_dir: &str
    ) -> Result<PathBuf,
        AppError> {
        tracing::info!("Fetching RSS feed from: {}", feed_url);

        let channel = self.construct_rss_channel(feed_url).await?;

        // Extract the most recent episode and its audio URL
        let latest_item = channel.items().first()
            .ok_or_else(|| AppError::InternalServerError("No episodes found in feed".into()))?;

        let enclosure = latest_item.enclosure()
            .ok_or_else(|| AppError::InternalServerError("No audio enclosure found".into()))?;

        let audio_url = enclosure.url();

        let title = latest_item.title().expect("No Title");

        let guid = latest_item.guid().expect("No GUID").value().to_string();

        // Create a safe filename from the episode title
        let raw_title = latest_item.title().unwrap_or("unknown_episode");
        let safe_title = raw_title.replace(|c: char| !c.is_alphanumeric(), "_");
        let file_name = format!("{}.mp3", safe_title);
        let file_path = Path::new(output_dir).join(&file_name);

        let file_path = self._download_episode(audio_url, title, &guid).await
            .map_err(|_| AppError::InternalServerError("Failed to download audio".into()))?;

        Ok(file_path)
    }
}