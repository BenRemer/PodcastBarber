use std::path::PathBuf;
use axum::Router;
use tower_http::trace::TraceLayer;
use crate::state::AppState;
use crate::constants::{BASE_DOWNLOAD_PATH, DATABASE_URL, DEFAULT_WHISPER_URL};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::storage::manager::DownloadManager;
use crate::storage::database::Database;

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

    // Database
    let db = Database::connect(&database_url).await.expect("Failed connectin to database");
    sqlx::migrate!("./migrations").run(&db.pool).await.expect("Failed migrate from database");


    let download_manager = DownloadManager::new(PathBuf::from(base_download_path));
    let state = AppState::new(db, download_manager, whisper_url);

    let app = Router::new()
        .nest("/api", routes::api_router(state))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
