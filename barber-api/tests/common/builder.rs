use barber_api::models::episode::{Episode, EpisodeState};
use barber_api::models::podcast::Podcast;
use barber_api::processors::coordinator::AudioCoordinator;
use barber_api::services::detection::DetectionService;
use barber_api::services::episode::EpisodeService;
use barber_api::services::podcast::PodcastService;
use barber_api::services::rss::RSSFeedService;
use barber_api::services::transcribe::TranscribeService;
use barber_api::storage::download::DownloadManager;
use barber_api::storage::repository::episode::EpisodeRepository;
use barber_api::storage::repository::podcast::PodcastRepository;
use barber_api::utils::{generate_episode_uuid, generate_podcast_uuid};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use wiremock::MockServer;
use crate::common::context::TestContext;
use crate::common::mocks;

pub struct TestContextBuilder {
    start_workers: bool,
    whisper_url: Option<String>,
}

impl TestContextBuilder {
    pub fn new() -> Self {
        Self {
            start_workers: false,
            whisper_url: None,
        }
    }

    pub fn with_workers(mut self) -> Self {
        self.start_workers = true;
        self
    }

    pub fn whisper_url(mut self, url: impl Into<String>) -> Self {
        self.whisper_url = Some(url.into());
        self
    }

    pub async fn build(self) -> TestContext {
        let mock_server = MockServer::start().await;
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let podcast_repository = PodcastRepository::new(pool.clone());
        let episode_repository = EpisodeRepository::new(pool.clone());

        let http = reqwest::Client::new();

        let (download_tx, download_rx) = mpsc::channel(100);
        let (transcribe_tx, transcribe_rx) = mpsc::channel(100);
        let (detect_tx, detect_rx) = mpsc::channel(100);

        let (download, download_worker) = DownloadManager::new(
            tempfile::tempdir().unwrap().path().into(),
            http.clone(),
            download_tx,
            100,
        );
        let download = Arc::new(download);
        let rss_service = RSSFeedService::new(http.clone());
        let podcast_service = PodcastService::new(podcast_repository.clone());
        let episode_service = EpisodeService::new(episode_repository.clone(), download.clone());
        let whisper_url = self
            .whisper_url
            .unwrap_or("http://whisper_sidecar:8000/v1".into());
        let (whisper, whisper_worker) =
            TranscribeService::new(whisper_url, http.clone(), transcribe_tx, 100);
        let whisper = Arc::new(whisper);
        let (detection, detection_worker) = DetectionService::new(detect_tx, 100);
        let detection = Arc::new(detection);
        let coordinator = AudioCoordinator::new(
            download_rx,
            whisper.clone(),
            transcribe_rx,
            detection.clone(),
            detect_rx,
            episode_repository.clone(),
        );

        let mut ctx = TestContext {
            mock_server,
            pool,
            podcast_repository,
            episode_repository,
            podcast_service,
            episode_service,
            rss_service,
            detection_service: detection,
            audio_coordinator: Some(coordinator),
            download_worker: Some(download_worker),
            whisper_worker: Some(whisper_worker),
            detection_worker: Some(detection_worker),
        };

        if self.start_workers {
            if let Some(w) = ctx.download_worker.take() {
                tokio::spawn(w.run());
            }
            if let Some(w) = ctx.whisper_worker.take() {
                tokio::spawn(w.run());
            }
            if let Some(w) = ctx.detection_worker.take() {
                tokio::spawn(w.run());
            }
            if let Some(w) = ctx.audio_coordinator.take() {
                tokio::spawn(w.run());
            }
        }

        ctx
    }
}

pub struct PodcastFixtureBuilder<'a> {
    ctx: &'a TestContext,

    title: String,
    subscribed: bool,
    episodes: usize,
}

impl<'a> PodcastFixtureBuilder<'a> {
    pub fn new(ctx: &'a TestContext) -> Self {
        Self {
            ctx,
            title: "Fixture Podcast".into(),
            subscribed: false,
            episodes: 0,
        }
    }

    pub fn title(mut self, value: &str) -> Self {
        self.title = value.into();
        self
    }

    pub fn subscribed(mut self) -> Self {
        self.subscribed = true;
        self
    }

    pub fn episodes(mut self, count: usize) -> Self {
        self.episodes = count;
        self
    }

    pub async fn build(self) -> (Podcast, Vec<Episode>) {
        let feed_url = mocks::rss::create_feed(&self.ctx.mock_server, "feed.xml").await;
        let id = generate_podcast_uuid(&feed_url);
        let mut podcast = Podcast {
            id,
            title: self.title,
            feed_url,
            image_url: None,
            description: None,
            author: None,
        };
        if self.subscribed {
            podcast = self
                .ctx
                .podcast_service
                .subscribe_podcast(podcast)
                .await
                .unwrap();
        }

        let mut episodes = Vec::new();
        for _ in 0..self.episodes {
            episodes.push(
                EpisodeFixtureBuilder::new(self.ctx)
                    .podcast(podcast.clone())
                    .build()
                    .await,
            );
        }

        (podcast, episodes)
    }
}

pub struct EpisodeFixtureBuilder<'a> {
    ctx: &'a TestContext,

    podcast: Option<Podcast>,
    state: EpisodeState,
    title: String,
}

impl<'a> EpisodeFixtureBuilder<'a> {
    pub fn new(ctx: &'a TestContext) -> Self {
        Self {
            ctx,
            podcast: None,
            state: EpisodeState::Pending,
            title: "Fixture Episode".into(),
        }
    }

    pub fn podcast(mut self, podcast: Podcast) -> Self {
        self.podcast = Some(podcast);
        self
    }

    pub async fn build(self) -> Episode {
        let podcast = self.podcast.unwrap();

        let audio = mocks::audio::create(&self.ctx.mock_server).await;
        let episode = Episode {
            id: generate_episode_uuid(podcast.id, &Uuid::new_v4().to_string()),
            podcast_id: podcast.id,
            guid: Uuid::new_v4().to_string(),
            title: self.title,
            audio_url: audio,
            local_file_path: None,
            state: self.state,
            transcript: None,
        };
        self.ctx
            .episode_repository
            .upsert(episode.clone())
            .await
            .unwrap();

        episode
    }
}
