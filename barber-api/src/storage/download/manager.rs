use crate::error::AppError;
use crate::storage::download::core::DownloadCore;
use crate::storage::download::{DownloadJob, DownloadResult, DownloadWorker};
use reqwest::Client;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct DownloadManager {
    job_queue: mpsc::Sender<DownloadJob>,
}

impl DownloadManager {
    pub fn new(
        base_dir: PathBuf,
        client: Client,
        download_finished_callback: mpsc::Sender<DownloadResult>,
        buffer: usize,
    ) -> (Self, DownloadWorker) {
        let (download_job_sender, download_job_receiver) = mpsc::channel::<DownloadJob>(buffer);
        let service = Self {
            job_queue: download_job_sender,
        };
        let worker = DownloadWorker {
            core: DownloadCore::new(base_dir, client),
            queue_receive: download_job_receiver,
            callback: download_finished_callback,
        };
        (service, worker)
    }

    pub async fn enqueue_download(&self, job: DownloadJob) -> Result<(), AppError> {
        tracing::info!("Enqueueing downloaded job {}", job.tracking_id);
        println!("Starting download enqueue");
        self.job_queue.send(job).await.map_err(|e| {
            tracing::error!("Download queue rejected job: {}", e);
            AppError::InternalServerError("Download queue is full or offline".into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::sync::mpsc;
    use uuid::Uuid;

    fn create_dummy_job() -> DownloadJob {
        DownloadJob {
            tracking_id: Uuid::new_v4(),
            audio_url: "http://example.com/audio.mp3".to_string(),
            folder_name: "Test Podcast".to_string(),
            guid: "test-guid-123".to_string(),
        }
    }

    #[tokio::test]
    async fn test_enqueue_download_success() {
        let (callback_tx, _callback_rx) = mpsc::channel(10);
        let client = reqwest::Client::new();
        let (manager, mut worker) =
            DownloadManager::new(PathBuf::from("/tmp"), client, callback_tx, 10);
        let job = create_dummy_job();
        let expected_id = job.tracking_id;
        let result = manager.enqueue_download(job).await;
        assert!(
            result.is_ok(),
            "Manager should successfully enqueue the job"
        );
        let received_job = worker
            .queue_receive
            .recv()
            .await
            .expect("Worker should have received a job");
        assert_eq!(received_job.tracking_id, expected_id);
    }

    #[tokio::test]
    async fn test_enqueue_fails_when_worker_is_offline() {
        let (callback_tx, _callback_rx) = mpsc::channel(10);
        let client = reqwest::Client::new();
        let (manager, worker) =
            DownloadManager::new(PathBuf::from("/tmp"), client, callback_tx, 10);
        // Simulate a catastrophic crash: drop the worker so the receiver channel closes!
        drop(worker);
        let result = manager.enqueue_download(create_dummy_job()).await;
        assert!(
            result.is_err(),
            "Manager should fail if the queue is closed"
        );
        assert!(matches!(
            result.unwrap_err(),
            AppError::InternalServerError(_)
        ));
    }
}
