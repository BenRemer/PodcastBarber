use std::path::PathBuf;
use std::sync::Arc;
use axum::Router;
use tokio::sync::mpsc;
use tower_http::trace::TraceLayer;
use crate::state::AppState;
use crate::constants::{BASE_DOWNLOAD_PATH, DATABASE_URL, DEFAULT_BUFFER_QUEUE, DEFAULT_WHISPER_URL};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::services::episode::EpisodeService;
use crate::services::podcast::PodcastService;
use crate::services::rss::RSSFeedService;
use crate::services::whisper::WhisperService;
use crate::storage::database::Database;
use crate::storage::download::{DownloadManager, DownloadResult};

pub mod error;
pub mod routes;
pub mod state;
pub mod services;
pub mod constants;
pub mod extractors;
pub mod storage;
pub mod utils;
pub mod models;

pub async fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,barber_api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Env
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| String::from(DATABASE_URL));
    let whisper_url = std::env::var("WHISPER_BASE_URL")
        .unwrap_or_else(|_| String::from(DEFAULT_WHISPER_URL));
    let base_download_path = std::env::var("BASE_DOWNLOAD_PATH")
        .unwrap_or_else(|_| String::from(BASE_DOWNLOAD_PATH));
    let download_queue_size = std::env::var("DOWNLOAD_QUEUE")
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(DEFAULT_BUFFER_QUEUE);

    // Database
    let db = Database::connect(&database_url).await.expect("Failed connectin to database");
    sqlx::migrate!("./migrations").run(&db.pool).await.expect("Failed migrate from database");

    // External Infra
    let http_client = reqwest::Client::new();

    // Background workers
    let (download_callback_sender, download_callback_receiver) =
        mpsc::channel::<DownloadResult>(download_queue_size);

    // Services
    let (manager_handle, manager_worker) = DownloadManager::new(
        PathBuf::from(base_download_path),
        http_client.clone(),
        download_callback_sender,
        download_queue_size
    );
    let download_manager = Arc::new(manager_handle);
    let whisper_service = WhisperService::new(
        whisper_url,
        http_client.clone()
    );
    let rss_service = RSSFeedService::new(
        http_client.clone()
    );
    let podcast_service = PodcastService::new(
        db.podcast_repository()
    );
    let (episode_service, episode_worker) = EpisodeService::new(
        db.episode_repository(),
        Arc::clone(&download_manager),
        download_callback_receiver
    );

    // Spawn background workers
    tokio::spawn(async move {
        manager_worker.run().await;
    });
    tokio::spawn(async move {
        episode_worker.run().await;
    });

    // Set state
    let state = AppState::new(
        Arc::from(whisper_service),
        Arc::from(rss_service),
        Arc::from(podcast_service),
        Arc::from(episode_service)
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
