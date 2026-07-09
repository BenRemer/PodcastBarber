use tokio::sync::mpsc;
use crate::storage::download::core::DownloadCore;
use crate::storage::download::types::{DownloadJob, DownloadResult};

pub struct DownloadWorker {
    pub(crate) core: DownloadCore,
    pub(crate) queue_receive: mpsc::Receiver<DownloadJob>,
    pub(crate) callback: mpsc::Sender<DownloadResult>,
}

impl DownloadWorker {
    pub async fn run(mut self) {
        while let Some(job) = self.queue_receive.recv().await {
            let status = match self.core.download_to_path(&job.audio_url, &job.podcast_title, &job
                .guid).await {
                Ok(path) => Ok(path),
                Err(e) => Err(e),
            };

            let _ = self.callback.send(DownloadResult {
                id: job.episode_id,
                status
            }).await;
        }
    }
}