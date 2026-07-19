use std::sync::Arc;

use sqlx::SqlitePool;
use wiremock::MockServer;

use barber_api::processors::coordinator::AudioCoordinator;

use barber_api::services::episode::EpisodeService;
use barber_api::services::podcast::PodcastService;
use barber_api::services::rss::RSSFeedService;
use barber_api::services::{
    detection::{DetectionService, DetectionWorker},
    transcribe::TranscribeWorker,
};
use barber_api::storage::download::DownloadWorker;

use crate::common::builder::TestContextBuilder;
use barber_api::storage::repository::detection::DetectionRepository;
use barber_api::storage::repository::transcript::TranscriptRepository;
use barber_api::storage::repository::{episode::EpisodeRepository, podcast::PodcastRepository};

pub struct TestContext {
    pub mock_server: MockServer,

    pub pool: SqlitePool,

    pub podcast_repository: PodcastRepository,
    pub episode_repository: EpisodeRepository,
    pub transcript_repository: TranscriptRepository,
    pub detection_repository: DetectionRepository,

    pub podcast_service: PodcastService,
    pub episode_service: EpisodeService,
    pub rss_service: RSSFeedService,

    pub detection_service: Arc<DetectionService>,

    pub audio_coordinator: Option<AudioCoordinator>,
    pub download_worker: Option<DownloadWorker>,
    pub whisper_worker: Option<TranscribeWorker>,
    pub detection_worker: Option<DetectionWorker>,
}

impl TestContext {
    pub fn builder() -> TestContextBuilder {
        TestContextBuilder::new()
    }
}
