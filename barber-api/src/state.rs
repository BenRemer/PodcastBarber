use std::sync::Arc;
use crate::services::rss::RSSFeedService;
use crate::services::podcast::PodcastService;
use crate::services::whisper::WhisperService;
use crate::storage::database::Database;
use crate::storage::manager::DownloadManager;

#[derive(Clone)]
pub struct AppState {
    pub whisper_service: Arc<WhisperService>,
    pub rssfeed_service: Arc<RSSFeedService>,
    pub podcast_service: Arc<PodcastService>,
}

impl AppState {
    pub fn new(
        db: Database,
        download_manager: DownloadManager,
        whisper_url: String
    ) -> Self {
        let shared_manager = Arc::new(download_manager);

        let whisper_service = WhisperService::new(whisper_url);
        let rssfeed_service = RSSFeedService::new();

        let podcast_service = PodcastService::new(
            db.podcast_repository(),
            rssfeed_service.clone(),
            Arc::clone(&shared_manager)
        );

        Self {
            whisper_service: Arc::new(whisper_service),
            rssfeed_service: Arc::new(rssfeed_service),
            podcast_service: Arc::new(podcast_service),
        }
    }
}