use crate::storage::download::core::Downloader;
use crate::storage::download::types::{DownloadJob, DownloadResult};
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};

pub struct DownloadWorker {
    pub(crate) core: Arc<dyn Downloader>,
    pub(crate) queue_receive: mpsc::Receiver<DownloadJob>,
    pub(crate) callback: mpsc::Sender<DownloadResult>, // audio coordinator callback
    pub(crate) concurrency_limit: usize,
}

impl DownloadWorker {
    pub async fn run(mut self) {
        tracing::info!("DownloadWorker background download worker starting...");

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));

        while let Some(job) = self.queue_receive.recv().await {
            tracing::info!("DownloadWorker received job: {:?}", job);

            let core_clone = self.core.clone();
            let callback_clone = self.callback.clone();

            // Wait for an available slot before spawning.
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            tokio::spawn(async move {
                let status = core_clone
                    .download_to_path(&job.audio_url, &job.folder_name, &job.guid)
                    .await;

                if let Err(e) = callback_clone
                    .send(DownloadResult {
                        tracking_id: job.tracking_id,
                        status,
                    })
                    .await
                {
                    tracing::error!("Failed to send download result to coordinator: {}", e);
                } else {
                    tracing::info!("Called download worker task to coordinator");
                }
                drop(permit);
            });
        }
        tracing::info!("DownloadWorker background download task shut down.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use async_trait::async_trait;
    use std::path::PathBuf;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time::Instant;
    use uuid::Uuid;

    struct MockCore;

    #[async_trait]
    impl Downloader for MockCore {
        async fn download_to_path(
            &self,
            _url: &str,
            _folder: &str,
            _guid: &str,
        ) -> Result<PathBuf, AppError> {
            // Simulate network delay
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok(PathBuf::from("/fake/success/path.mp3"))
        }
    }

    #[tokio::test]
    async fn test_worker_processes_concurrently() {
        let concurrency_limit = 3;
        let (job_tx, job_rx) = mpsc::channel(10);
        let (result_tx, mut result_rx) = mpsc::channel(10);

        let worker = DownloadWorker {
            queue_receive: job_rx,
            callback: result_tx,
            core: Arc::new(MockCore) as Arc<dyn Downloader>,
            concurrency_limit,
        };

        tokio::spawn(worker.run());

        for i in 0..concurrency_limit {
            let job = DownloadJob {
                tracking_id: Uuid::new_v4(),
                audio_url: format!("http://test.com/{}", i),
                folder_name: "test_folder".to_string(),
                guid: format!("guid-{}", i),
            };
            job_tx.send(job).await.unwrap();
        }

        // Drop the sender so the worker's `while let Some(...)` loop knows to terminate
        // once it finishes processing the queue.
        drop(job_tx);

        let start_time = Instant::now();
        let mut results = Vec::new();

        // Await the exact number of results we expect (concurrency_limit)
        for _ in 0..concurrency_limit {
            if let Some(result) = result_rx.recv().await {
                results.push(result);
            }
        }

        let elapsed = start_time.elapsed();

        assert_eq!(
            results.len(),
            concurrency_limit,
            "Did not receive all results"
        );

        // If it ran sequentially, it would take 3 * 200ms = 600ms.
        // If it ran concurrently, it should take ~200ms (plus minor Tokio overhead).
        assert!(
            elapsed < Duration::from_millis(400),
            "Worker is running sequentially! It took {:?}",
            elapsed
        );
    }
}
