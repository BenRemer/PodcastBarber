use std::path::PathBuf;
use std::sync::Arc;
use axum::Router;
use tower_http::trace::TraceLayer;
use crate::state::AppState;
use crate::constants::{BASE_DOWNLOAD_PATH, DEFAULT_WHISPER_URL};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::storage::download_manager::DownloadManager;

pub mod error;
pub mod routes;
pub mod state;
pub mod services;
pub mod constants;
pub mod extractors;
pub mod storage;

pub async fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,barber_api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let whisper_url = std::env::var("WHISPER_BASE_URL")
        .unwrap_or_else(|_| String::from(DEFAULT_WHISPER_URL));

    let base_download_path = std::env::var("BASE_DOWNLOAD_PATH")
        .unwrap_or_else(|_| String::from(BASE_DOWNLOAD_PATH));

    let download_manager = DownloadManager::new(PathBuf::from(base_download_path));
    let state = AppState::new(download_manager, whisper_url);

    let app = Router::new()
        .nest("/api", routes::api_router(state))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}