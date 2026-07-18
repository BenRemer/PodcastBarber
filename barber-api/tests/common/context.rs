use std::sync::Arc;
use barber_api::models::episode::{Episode, EpisodeState};
use barber_api::models::podcast::Podcast;
use barber_api::utils::{generate_episode_uuid, generate_podcast_uuid};
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use testcontainers::core::{IntoContainerPort, WaitFor};
use barber_api::storage::repository::episode::EpisodeRepository;
use barber_api::storage::repository::podcast::PodcastRepository;
use tokio::sync::mpsc;
use barber_api::processors::coordinator::AudioCoordinator;
use barber_api::services::detection::{DetectionResult, DetectionService, DetectionWorker};
use barber_api::services::episode::EpisodeService;
use barber_api::services::podcast::PodcastService;
use barber_api::services::rss::RSSFeedService;
use barber_api::services::transcribe::{TranscribeResult, TranscribeService, TranscribeWorker};
use barber_api::storage::download::{DownloadManager, DownloadResult, DownloadWorker};
use sqlx::SqlitePool;
use testcontainers::runners::AsyncRunner;
use crate::common;

pub struct TestContext {
    pub detection_service: Arc<DetectionService>,
    pub episode_service: EpisodeService,
    pub podcast_service: PodcastService,
    pub rss_service: RSSFeedService,
    pub mock_server: MockServer,
    pub pool: SqlitePool,
    pub podcast_repository: PodcastRepository,
    pub episode_repository: EpisodeRepository,

    // Wrapped in Option so tests can take ownership and run them manually
    pub audio_coordinator: Option<AudioCoordinator>,
    pub download_worker: Option<DownloadWorker>,
    pub whisper_worker: Option<TranscribeWorker>,
    pub detection_worker: Option<DetectionWorker>,
}

pub struct TestContextBuilder {
    start_background_workers: bool,
    whisper_url: Option<String>,
}

impl TestContextBuilder {
    pub fn new() -> Self {
        Self {
            start_background_workers: false,
            whisper_url: None,
        }
    }

    /// Automatically spawn all background workers
    pub fn with_background_workers(mut self) -> Self {
        self.start_background_workers = true;
        self
    }

    /// Override the Whisper URL
    pub fn with_whisper_url(mut self, url: impl Into<String>) -> Self {
        self.whisper_url = Some(url.into());
        self
    }

    pub async fn build(self) -> TestContext {
        // EXTERNAL INFRASTRUCTURE
        let mock_server = MockServer::start().await;
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let http_client = reqwest::Client::new();

        // DB
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let podcast_repository = PodcastRepository::new(pool.clone());
        let episode_repository = EpisodeRepository::new(pool.clone());

        // CHANNELS
        let (result_tx, result_rx) = mpsc::channel::<DownloadResult>(100);
        let (transcribe_tx, transcribe_rx) = mpsc::channel::<TranscribeResult>(100);
        let (detection_tx, detection_rx) = mpsc::channel::<DetectionResult>(100);

        // SERVICES & WORKERS
        let (download_handle, download_worker) = DownloadManager::new(
            temp_dir.path().to_path_buf(),
            http_client.clone(),
            result_tx,
            100,
        );
        let download_manager = Arc::new(download_handle);

        let rss_service = RSSFeedService::new(http_client.clone());
        let podcast_service = PodcastService::new(podcast_repository.clone());
        let episode_service =
            EpisodeService::new(episode_repository.clone(), download_manager.clone());

        let whisper_base_url = self
            .whisper_url
            .unwrap_or_else(|| "http://whisper_sidecar:8000/v1".to_string());
        let (whisper_handle, whisper_worker) =
            TranscribeService::new(whisper_base_url, http_client.clone(), transcribe_tx, 100);
        let whisper_service = Arc::new(whisper_handle);

        let (detection_handle, detection_worker) = DetectionService::new(detection_tx, 100);
        let detection_service = Arc::new(detection_handle);

        // COORDINATOR
        let audio_coordinator = AudioCoordinator::new(
            result_rx,
            Arc::clone(&whisper_service),
            transcribe_rx,
            Arc::clone(&detection_service),
            detection_rx,
            episode_repository.clone(),
        );

        let mut ctx = TestContext {
            detection_service,
            episode_service,
            podcast_service,
            rss_service,
            mock_server,
            pool,
            podcast_repository,
            episode_repository,
            audio_coordinator: Some(audio_coordinator),
            download_worker: Some(download_worker),
            whisper_worker: Some(whisper_worker),
            detection_worker: Some(detection_worker),
        };

        // START WORKERS IF REQUESTED
        if self.start_background_workers {
            if let Some(worker) = ctx.download_worker.take() {
                tokio::spawn(async move {
                    worker.run().await;
                });
            }
            if let Some(worker) = ctx.whisper_worker.take() {
                tokio::spawn(async move {
                    worker.run().await;
                });
            }
            if let Some(worker) = ctx.detection_worker.take() {
                tokio::spawn(async move {
                    worker.run().await;
                });
            }
            if let Some(coordinator) = ctx.audio_coordinator.take() {
                tokio::spawn(async move {
                    coordinator.run().await;
                });
            }
        }

        ctx
    }
}

impl TestContext {
    const FEED_NAME: &'static str = "feed";
    const AUDIO_NAME: &'static str = "episode";

    pub fn builder() -> TestContextBuilder {
        TestContextBuilder::new()
    }

    pub async fn create_podcast(
        &self,
        title_override: Option<&str>,
        feed_override: Option<&str>,
    ) -> Podcast {
        let resolved_feed_url = match feed_override {
            Some(url) => url.to_string(),
            None => {
                self.create_xml_feed_url(&format!("{}.xml", Self::FEED_NAME))
                    .await
            }
        };
        Podcast {
            id: generate_podcast_uuid(&resolved_feed_url),
            title: title_override
                .unwrap_or("Default Fixture Podcast")
                .to_string(),
            feed_url: resolved_feed_url,
            image_url: Some("http://127.0.0.1/image.png".to_string()),
            description: Some("Generated by TestContext".to_string()),
            author: Some("Test Author".to_string()),
        }
    }

    pub async fn create_subscribed_podcast(
        &self,
        title_override: Option<&str>,
        feed_override: Option<&str>,
    ) -> Podcast {
        let podcast = self.create_podcast(title_override, feed_override).await;

        self.podcast_repository
            .upsert(podcast.clone())
            .await
            .expect("Failed to insert fixture podcast");

        podcast
    }

    /// Creates an episode. Automatically generates a parent podcast if one isn't provided.
    pub async fn create_test_episode(
        &self,
        parent_podcast: Option<Podcast>,
        title_override: Option<&str>,
        state_override: Option<EpisodeState>,
    ) -> Episode {
        //  Resolve the parent podcast
        let podcast = match parent_podcast {
            Some(p) => p,
            None => self.create_subscribed_podcast(None, None).await,
        };

        let episode_title = title_override.unwrap_or("Default Fixture Episode");
        let audio_url = self.mock_audio_download(&Self::AUDIO_NAME).await;
        let guid = Uuid::new_v4().to_string();

        // Generate the Episode
        let episode = Episode {
            id: generate_episode_uuid(podcast.id, &guid),
            podcast_id: podcast.id,
            guid,
            title: episode_title.to_string(),
            audio_url,
            local_file_path: None,
            state: state_override.unwrap_or(EpisodeState::Pending),
            transcript: None,
        };

        self.episode_repository
            .upsert(episode.clone())
            .await
            .expect("Failed to insert fixture episode");

        episode
    }

    // todo move to mock file
    pub async fn create_xml_feed_url(&self, asset_name: &str) -> String {
        let asset_path = common::get_asset_path(asset_name);
        let xml_bytes = tokio::fs::read(asset_path)
            .await
            .expect("Unable to read XML bytes");

        let realistic_response = ResponseTemplate::new(200)
            .insert_header("content-type", "application/rss+xml; charset=utf-8")
            .set_body_bytes(xml_bytes);

        let unique_path = format!("/{}-{}.xml", Self::FEED_NAME, Uuid::new_v4());

        Mock::given(method("GET"))
            .and(path(unique_path.clone()))
            .respond_with(realistic_response)
            .mount(&self.mock_server)
            .await;

        format!("{}{}", self.mock_server.uri(), unique_path)
    }

    pub async fn mock_audio_download(&self, base_name: &str) -> String {
        let mock_audio_bytes = vec![0u8; 1024]; // 1KB dummy episode file

        // Strip ".mp3" if it's already there, then add a unique UUID
        let clean_name = base_name.trim_end_matches(".mp3");
        let unique_path = format!("/{}-{}.mp3", clean_name, Uuid::new_v4());

        // Mount the mock to Wiremock using the exact unique path
        Mock::given(method("GET"))
            .and(path(unique_path.clone()))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(mock_audio_bytes))
            .mount(&self.mock_server)
            .await;

        // Return the full URL that the Downloader can fetch
        format!("{}{}", self.mock_server.uri(), unique_path)
    }

    pub async fn start_whisper_sidecar() -> (ContainerAsync<GenericImage>, String) {
        let whisper_image = GenericImage::new("fedirz/faster-whisper-server", "latest-cuda")
            .with_wait_for(WaitFor::message_on_stderr("Application startup complete"))
            .with_exposed_port(8000.tcp())
            // use tiny model for test
            .with_env_var("WHISPER__MODEL", "tiny");

        println!("Booting ephemeral Whisper container...");
        let container = whisper_image
            .start()
            .await
            .expect("Failed to start Whisper container");

        let host_port = container.get_host_port_ipv4(8000).await.unwrap();
        // let dynamic_url = format!("http://127.0.0.1:{}/v1/audio/transcriptions", host_port);
        let dynamic_url = format!("http://127.0.0.1:{}/v1", host_port);

        println!("Container ready! Bound to dynamic port: {}", host_port);

        (container, dynamic_url)
    }
}