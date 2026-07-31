use crate::error::AppError;
use crate::services::detection::Detection;
use crate::services::detection::chunker::DefaultChunker;
use crate::services::detection::core::{DetectionConfig, DetectionCore};
use crate::services::detection::types::{DetectionJob, DetectionResult};
use crate::services::detection::worker::DetectionWorker;
use crate::storage::repository::detection::DetectionStore;
use crate::storage::repository::transcript::TranscriptStore;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct DetectionService {
    job_queue: mpsc::Sender<DetectionJob>,
    detection_repository: Arc<dyn DetectionStore>,
}

impl DetectionService {
    pub fn new(
        transcript_repository: Arc<dyn TranscriptStore>,
        detection_repository: Arc<dyn DetectionStore>,
        callback: mpsc::Sender<DetectionResult>,
        buffer: usize,
        concurrency_limit: usize,
    ) -> (Self, DetectionWorker) {
        let (queue_send, queue_receive) = mpsc::channel::<DetectionJob>(buffer);
        // todo pass in
        let core = Arc::new(DetectionCore::new(DetectionConfig::default()));
        let chunker = Arc::new(DefaultChunker);

        let service = Self {
            job_queue: queue_send,
            detection_repository: detection_repository.clone(),
        };
        let worker = DetectionWorker {
            transcript_repository,
            detection_repository,
            core,
            chunker,
            concurrency_limit,
            job_queue: queue_receive,
            callback,
        };
        (service, worker)
    }

    pub async fn detect_ads(&self, job: DetectionJob) -> Result<(), AppError> {
        self.job_queue.send(job).await.map_err(|e| {
            tracing::error!("Detection queue rejected job: {}", e);
            AppError::InternalServerError("Detection queue is full or offline".into())
        })
    }

    pub async fn list_detections(&self, episode_id: &Uuid) -> Result<Detection, AppError> {
        self.detection_repository
            .get_detection_by_episode(episode_id)
            .await?
            .ok_or(AppError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::detection::ProcessedSegment;
    use async_trait::async_trait;
    use rand::{rng, Rng};

    struct MockDetectionStore {
        segments: Vec<ProcessedSegment>,
    }
    #[async_trait]
    impl DetectionStore for MockDetectionStore {
        async fn upsert(&self, _detection: Detection) -> Result<(), AppError> {
            Ok(())
        }

        async fn get_detection_by_episode(
            &self,
            episode_id: &Uuid,
        ) -> Result<Option<Detection>, AppError> {
            Ok(Some(Detection {
                episode_id: *episode_id,
                segments: self.segments.clone(),
            }))
        }
    }

    #[tokio::test]
    async fn test_list_detections() {
        let mut rng = rng();
        let mut segments = Vec::new();

        for i in 0..5 {
            segments.push(ProcessedSegment {
                start_time: (i * 10) as f64,
                end_time: (i + 10) as f64,
                text: "".to_string(),
                ad_score: rng.random_range(1..=100),
                is_ad: if i == 0 {true} else {false},
            })
        }

        let store = Arc::new(MockDetectionStore {
            segments,
        }) as Arc<dyn DetectionStore>;

        let (queue_send, _queue_receive) = mpsc::channel::<DetectionJob>(10);

       let service = DetectionService {
           job_queue: (queue_send),
           detection_repository: store.clone(),
       };

        let detection = service.list_detections(&Uuid::new_v4()).await.unwrap();
        assert_eq!(detection.segments.len(), 5);
        assert_eq!(detection.segments.iter().filter(|seg| seg.is_ad).count(), 1)
    }
}
