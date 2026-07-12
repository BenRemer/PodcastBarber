use crate::storage::download::core::DownloadCore;
use crate::storage::download::types::{DownloadJob, DownloadResult};
use tokio::sync::mpsc;

pub struct DownloadWorker {
    pub(crate) core: DownloadCore,
    pub(crate) queue_receive: mpsc::Receiver<DownloadJob>,
    pub(crate) callback: mpsc::Sender<DownloadResult>,
}

impl DownloadWorker {
    pub async fn run(mut self) {
        tracing::info!("DownloadWorker background download worker starting...");
        while let Some(job) = self.queue_receive.recv().await {
            tracing::info!("DownloadWorker received job: {:?}", job);
            let status = self
                .core
                .download_to_path(&job.audio_url, &job.folder_name, &job.guid)
                .await;

            if let Err(e) = self
                .callback
                .send(DownloadResult {
                    tracking_id: job.tracking_id,
                    status,
                })
                .await
            {
                tracing::error!("Failed to send download result to coordinator: {}", e);
            } else {
                tracing::info!("Called download worker task to coordinator");
            };
        }
        tracing::info!("DownloadWorker background download task shut down.");
    }
}
