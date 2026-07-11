use crate::models::episode::EpisodeState;
use crate::storage::download::DownloadResult;
use crate::storage::repository::episode::EpisodeRepository;
use tokio::sync::mpsc;

pub struct EpisodeWorker {
    pub(crate) repo: EpisodeRepository,
    pub(crate) download_callback: mpsc::Receiver<DownloadResult>,
}

impl EpisodeWorker {
    pub async fn run(mut self) {
        tracing::info!("EpisodeService background state worker starting...");
        while let Some(result) = self.download_callback.recv().await {
            if let Ok(Some(mut episode)) = self.repo.get(&result.id).await {
                match result.status {
                    Ok(path) => {
                        tracing::info!("Recording success for episode {}", result.id);
                        episode.state = EpisodeState::Downloaded;
                        episode.local_file_path = Some(path.to_string_lossy().into_owned());
                    }
                    Err(e) => {
                        tracing::error!("Recording failure for episode {}: {:?}", result.id, e);
                        episode.state = EpisodeState::Error;
                    }
                }
                let _ = self.repo.upsert(episode).await;
            }
        }
        tracing::info!("EpisodeService state worker shut down.");
    }
}
