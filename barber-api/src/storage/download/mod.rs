mod manager;
mod core;
mod worker;
mod types;

pub use manager::DownloadManager;
pub use worker::DownloadWorker;
pub use types::{DownloadResult, DownloadJob};