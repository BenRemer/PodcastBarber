use std::sync::Arc;
use crate::services::rss_feed::RSSFeedService;
use crate::services::whisper::WhisperService;
use crate::storage::download_manager::DownloadManager;

#[derive(Clone)]
pub struct AppState {
    pub whisper_service: WhisperService,
    pub rssfeed_service: RSSFeedService,
    pub download_manager: Arc<DownloadManager>,
}

impl AppState {
    pub fn new(
        download_manager: DownloadManager,
        whisper_url: String
    ) -> Self {
        let shared_manager = Arc::new(download_manager);
        Self {
            whisper_service: WhisperService::new(whisper_url),
            rssfeed_service: RSSFeedService::new(Arc::clone(&shared_manager)),
            download_manager: Arc::clone(&shared_manager),
        }
    }
}