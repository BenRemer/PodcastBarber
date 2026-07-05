use std::sync::Arc;
use crate::services::rss::RSSFeedService;
use crate::services::podcast::PodcastService;
use crate::services::whisper::WhisperService;
use crate::storage::database::Database;
use crate::storage::manager::DownloadManager;

#[derive(Clone)]
pub struct AppState {
    pub whisper_service: WhisperService,
    pub rssfeed_service: RSSFeedService,
    pub podcast_service: PodcastService,
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
            Arc::clone(&shared_manager) // todo clone instead of arc?
        );

        Self {
            whisper_service,
            rssfeed_service,
            podcast_service,
        }
    }
}