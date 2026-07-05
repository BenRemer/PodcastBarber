use std::path::PathBuf;
use crate::error::AppError;

#[derive(Clone)]
pub struct DownloadManager {
    base_dir: PathBuf,
}

impl DownloadManager {
    const PODCAST_DIR: &str = "podcast";

    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    // Sanitizes a string for use as a folder name
    fn sanitize_name(&self, name: &str) -> String {
        name.replace(|c: char| !c.is_alphanumeric() && c != ' ', "_") // todo do i want spaces
            .trim()
            .to_string()
    }

    // Returns the full PathBuf and ensures the folder exists
    pub async fn prepare_episode_path(
        &self, podcast_name: &str, episode_guid: &str
    )-> Result<PathBuf, std::io::Error> {
        let folder_name = self.sanitize_name(podcast_name);
        let podcast_dir = self.base_dir.join(Self::PODCAST_DIR).join(folder_name);

        // Create the podcast-specific folder if it doesn't exist
        if !podcast_dir.exists() {
            tokio::fs::create_dir_all(&podcast_dir).await?;
        }

        Ok(podcast_dir.join(format!("{}.mp3", episode_guid)))
    }

    pub async fn download_to_path(
        &self,
        client: &reqwest::Client,
        audio_url: &str,
        podcast_name: &str,
        guid: &str,
    ) -> Result<PathBuf, AppError> {
        // Prepare path
        let output_path = self.prepare_episode_path(podcast_name, guid).await
            .map_err(|e| {
                tracing::error!("Storage error: {:?}", e);
                AppError::InternalServerError("Failed to prepare storage".into())
            })?;

        // if file already exists return it
        if output_path.exists() {
            tracing::info!("Episode already downloaded.");
            return Ok(output_path);
        }

        // Fetch and Stream
        let mut response = client.get(audio_url).send().await
            .map_err(|e| {
                tracing::error!("Failed to start download: {}", e);
                AppError::InternalServerError("Audio download failed".into())
            })?;

        let mut file = tokio::fs::File::create(&output_path).await
            .map_err(|e| {
                tracing::error!("Failed to create file: {:?}", e);
                AppError::InternalServerError("Failed to create local file".into())
            })?;

        while let Some(chunk) = response.chunk().await.map_err(|_| {
            AppError::InternalServerError("Interrupted while streaming".into())
        })? {
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await
                .map_err(|_| AppError::InternalServerError("Failed to write to disk".into()))?;
        }

        Ok(output_path)
    }
}