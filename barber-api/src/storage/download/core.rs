use std::path::PathBuf;
use reqwest::Client;
use crate::error::AppError;

#[derive (Clone)]
pub struct DownloadCore {
    pub(crate) base_dir: PathBuf,
    pub(crate) client: Client,
}

impl DownloadCore {
    const PODCAST_DIR: &str = "podcast";

    pub fn new(
        base_dir: PathBuf,
        client: Client,
    ) -> Self {
        Self { base_dir, client }
    }

    // Returns the full PathBuf and ensures the folder exists
    pub async fn prepare_episode_path(
        &self, folder_name: &str, episode_guid: &str
    )-> Result<PathBuf, std::io::Error> {
        let podcast_dir = self.base_dir.join(Self::PODCAST_DIR).join(folder_name);

        // Create the podcast-specific folder if it doesn't exist
        if !podcast_dir.exists() {
            tokio::fs::create_dir_all(&podcast_dir).await?;
        }

        Ok(podcast_dir.join(format!("{}.mp3", episode_guid)))
    }

    pub async fn download_to_path(
        &self,
        audio_url: &str,
        folder_name: &str,
        guid: &str,
    ) -> Result<PathBuf, AppError> {
        // Prepare path
        let output_path = self.prepare_episode_path(folder_name, guid).await
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
        let mut response = self.client
            .get(audio_url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to download {}: {:?}", audio_url, e);
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

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use tempfile::tempdir;
    use tokio::io::AsyncReadExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_prepare_episode_path_creates_directories() {
        let temp_dir = tempdir().unwrap();
        let client = Client::new();
        let core = DownloadCore::new(temp_dir.path().to_path_buf(), client);
        let podcast_name = "Tech_Talk: 101";
        let guid = "episode-12345";
        let output_path = core
            .prepare_episode_path(podcast_name, guid)
            .await
            .unwrap();
        let expected_folder = temp_dir.path().join("podcast").join("Tech_Talk_ 101");
        assert!(
            expected_folder.exists(),
            "The podcast specific directory should be created"
        );
        assert_eq!(
            output_path.file_name().unwrap().to_str().unwrap(),
            "episode-12345.mp3"
        );
    }

    #[tokio::test]
    async fn test_download_to_path_skips_existing_file() {
        let temp_dir = tempdir().unwrap();
        let client = Client::new();
        let core = DownloadCore::new(temp_dir.path().to_path_buf(), client);
        let output_path = core
            .prepare_episode_path("Existing Podcast", "guid-999")
            .await
            .unwrap();
        tokio::fs::write(&output_path, b"already downloaded data")
            .await
            .unwrap();
        let result = core
            .download_to_path("http://invalid-url.local", "Existing Podcast", "guid-999")
            .await
            .unwrap();
        assert_eq!(result, output_path);
        let contents = tokio::fs::read_to_string(&output_path).await.unwrap();
        assert_eq!(contents, "already downloaded data");
    }

    #[tokio::test]
    async fn test_download_to_path_success() {
        let mock_server = MockServer::start().await;
        let fake_audio_bytes = b"fake-mp3-audio-data-stream";
        Mock::given(method("GET"))
            .and(path("/audio.mp3"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(fake_audio_bytes.as_slice()))
            .mount(&mock_server)
            .await;
        let temp_dir = tempdir().unwrap();
        let client = Client::new();
        let core = DownloadCore::new(temp_dir.path().to_path_buf(), client);
        let download_url = format!("{}/audio.mp3", mock_server.uri());
        let result_path = core
            .download_to_path(&download_url, "Valid Podcast", "guid-777")
            .await
            .expect("Download should succeed");
        assert!(result_path.exists(), "The audio file must exist on disk");
        let mut file = tokio::fs::File::open(&result_path).await.unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.unwrap();
        assert_eq!(
            buffer, fake_audio_bytes,
            "Downloaded file contents should match the server payload"
        );
    }
}
