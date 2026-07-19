use crate::constants::{
    BASE_DOWNLOAD_PATH, DATABASE_URL, DEFAULT_BUFFER_QUEUE, DEFAULT_WHISPER_URL,
};
use crate::processors::coordinator::AudioCoordinator;
use crate::services::detection::{DetectionResult, DetectionService};
use crate::services::episode::EpisodeService;
use crate::services::podcast::PodcastService;
use crate::services::rss::RSSFeedService;
use crate::services::transcribe::TranscribeService;
use crate::services::transcribe::types::TranscribeResult;
use crate::state::AppState;
use crate::storage::database::Database;
use crate::storage::download::{DownloadManager, DownloadResult};
use axum::Router;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod constants;
pub mod error;
pub mod extractors;
pub mod models;
pub mod processors;
pub mod routes;
pub mod services;
pub mod state;
pub mod storage;
pub mod utils;

pub async fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,barber_api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Env
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| String::from(DATABASE_URL));
    let whisper_url =
        std::env::var("WHISPER_BASE_URL").unwrap_or_else(|_| String::from(DEFAULT_WHISPER_URL));
    let base_download_path =
        std::env::var("BASE_DOWNLOAD_PATH").unwrap_or_else(|_| String::from(BASE_DOWNLOAD_PATH));
    let download_queue_size = std::env::var("DOWNLOAD_QUEUE")
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(DEFAULT_BUFFER_QUEUE);
    // todo set size
    let transcribe_size = 20;
    let detection_size = 20;

    // Database
    let db = Database::connect(&database_url)
        .await
        .expect("Failed connectin to database");
    sqlx::migrate!("./migrations")
        .run(&db.pool)
        .await
        .expect("Failed migrate from database");

    // External Infra
    let http_client = reqwest::Client::new();

    // Background workers
    let (download_callback_sender, download_callback_receiver) =
        mpsc::channel::<DownloadResult>(download_queue_size);
    let (transcribe_result_sender, transcribe_result_receiver) =
        mpsc::channel::<TranscribeResult>(transcribe_size);
    let (detection_result_sender, detection_result_receiver) =
        mpsc::channel::<DetectionResult>(detection_size);

    // Services
    let (download_handle, download_worker) = DownloadManager::new(
        PathBuf::from(base_download_path),
        http_client.clone(),
        download_callback_sender,
        download_queue_size,
    );
    let download_manager = Arc::new(download_handle);
    let (whisper_handle, whisper_worker) = TranscribeService::new(
        whisper_url,
        http_client.clone(),
        transcribe_result_sender,
        transcribe_size,
        db.transcript_repository(),
    );
    let whisper_service = Arc::new(whisper_handle);
    let rss_service = RSSFeedService::new(http_client.clone());
    let podcast_service = PodcastService::new(db.podcast_repository());
    let episode_service =
        EpisodeService::new(db.episode_repository(), Arc::clone(&download_manager));
    let (detection_handle, detection_worker) = DetectionService::new(
        db.transcript_repository(),
        db.detection_repository(),
        detection_result_sender,
        detection_size,
    );
    let detection_service = Arc::new(detection_handle);

    // Processors
    let audio_processor = AudioCoordinator::new(
        download_callback_receiver,
        Arc::clone(&whisper_service),
        transcribe_result_receiver,
        Arc::clone(&detection_service),
        detection_result_receiver,
        db.episode_repository(),
        db.transcript_repository(),
        None,
    );

    // Spawn background workers
    tokio::spawn(async move {
        download_worker.run().await;
    });
    tokio::spawn(async move {
        whisper_worker.run().await;
    });
    tokio::spawn(async move {
        detection_worker.run().await;
    });
    tokio::spawn(async move {
        audio_processor.run().await;
    });

    // Set state
    let state = AppState::new(
        Arc::from(whisper_service),
        Arc::from(rss_service),
        Arc::from(podcast_service),
        Arc::from(episode_service),
        Arc::from(detection_service),
        // Arc::from(audio_processor),
    );

    // Routes
    let app = Router::new()
        .nest("/api", routes::api_router(state))
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());

    // Start
    axum::serve(listener, app).await.unwrap();
}
