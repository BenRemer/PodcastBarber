use crate::services::detection::core::DetectionCore;
use crate::services::detection::types::{DetectionJob, DetectionResult};
use tokio::sync::mpsc;

pub struct DetectionWorker {
    pub core: DetectionCore,
    pub job_queue: mpsc::Receiver<DetectionJob>,
    pub callback: mpsc::Sender<DetectionResult>,
}

impl DetectionWorker {
    pub async fn run(mut self) {
        tracing::info!("Starting detection worker...");
        while let Some(job) = self.job_queue.recv().await {
            tracing::info!("Received job: {:?}", job);
            let _id = job.tracking_id;
            // get transcript, generate chunks send through processing
            // let transcript =
            // let chunks = generate
        }
        tracing::info!("Detection worker ended.");
    }
}
