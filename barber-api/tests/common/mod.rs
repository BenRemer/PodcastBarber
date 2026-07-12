pub mod builder;

use crate::common;
use barber_api::models::episode::{Episode, EpisodeState};
use barber_api::models::podcast::Podcast;
use barber_api::processors::coordinator::AudioCoordinator;
use barber_api::services::detection::{DetectionResult, DetectionService};
use barber_api::services::episode::EpisodeService;
use barber_api::services::podcast::PodcastService;
use barber_api::services::rss::RSSFeedService;
use barber_api::services::transcribe::TranscribeResult;
use barber_api::services::transcribe::TranscribeService;
use barber_api::storage::download::{DownloadManager, DownloadResult};
use barber_api::storage::repository::episode::EpisodeRepository;
use barber_api::storage::repository::podcast::PodcastRepository;
use barber_api::utils::{generate_episode_uuid, generate_podcast_uuid};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use std::sync::Arc;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::fs;
use tokio::sync::mpsc;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub struct TestContext {
    pub detection_service: Arc<DetectionService>,
    pub episode_service: EpisodeService,
    pub podcast_service: PodcastService,
    pub rss_service: RSSFeedService,
    pub mock_server: MockServer,
    pub pool: SqlitePool,
    pub podcast_repository: PodcastRepository,
    pub episode_repository: EpisodeRepository,
    pub audio_coordinator: AudioCoordinator,
}

impl TestContext {
    const FEED_NAME: &'static str = "feed";
    const AUDIO_NAME: &'static str = "episode";

    // todo break this up into a builder
    pub async fn setup() -> Self {
        // EXTERNAL INFRASTRUCTURE (Network, Disk, DB)
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

        // BACKGROUND WORKERS & CHANNELS
        let (result_tx, result_rx) = mpsc::channel::<DownloadResult>(100);
        let (transcribe_result_sender, transcribe_result_receiver) =
            mpsc::channel::<TranscribeResult>(100);
        let (detection_result_sender, detection_result_receiver) =
            mpsc::channel::<DetectionResult>(100);

        // SERVICES
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
        let (whisper_handle, whisper_worker) = TranscribeService::new(
            "http://whisper_sidecar:8000/v1".to_string(),
            http_client.clone(),
            transcribe_result_sender,
            100,
        );
        let whisper_service = Arc::new(whisper_handle);

        let (detection_handle, detection_worker) =
            DetectionService::new(detection_result_sender, 100);
        let detection_service = Arc::new(detection_handle);

        // Processors
        let audio_coordinator = AudioCoordinator::new(
            result_rx,
            Arc::clone(&whisper_service),
            transcribe_result_receiver,
            Arc::clone(&detection_service),
            detection_result_receiver,
            episode_repository.clone(),
        );

        // START WORKERS
        tokio::spawn(async move {
            download_worker.run().await;
        });
        tokio::spawn(async move {
            whisper_worker.run().await;
        });
        tokio::spawn(async move {
            detection_worker.run().await;
        });

        // RETURN ASSEMBLED CONTEXT
        Self {
            detection_service,
            episode_service,
            podcast_service,
            rss_service,
            mock_server,
            pool,
            podcast_repository,
            episode_repository,
            audio_coordinator,
        }
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

    pub async fn start_whisper_sidecar(&self) -> (ContainerAsync<GenericImage>, String) {
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

pub fn get_asset_path(filename: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
        .join(filename)
}

pub async fn save_json_to_assets<T: Serialize>(filename: &str, data: &T) {
    let json_string =
        serde_json::to_string_pretty(data).expect("Failed to serialize data to string");

    let output_path = get_asset_path(filename);

    fs::write(&output_path, json_string)
        .await
        .expect(&format!("Failed to save JSON to disk at {:?}", output_path));
}

pub async fn read_json_from_assets<T: DeserializeOwned>(filename: &str) -> T {
    let input_path = get_asset_path(filename);

    let json_string = fs::read_to_string(&input_path).await.expect(&format!(
        "Failed to read JSON file from disk at {:?}",
        input_path
    ));

    serde_json::from_str(&json_string)
        .expect(&format!("Failed to parse JSON from {:?}", input_path))
}
