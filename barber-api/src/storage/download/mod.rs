mod core;
mod manager;
mod types;
mod worker;

pub use manager::DownloadManager;
pub use types::{DownloadJob, DownloadResult};
pub use worker::DownloadWorker;
