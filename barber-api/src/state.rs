use std::path::PathBuf;
use std::sync::Arc;
use crate::services::rss::RSSFeedService;
use crate::services::podcast::PodcastService;
use crate::services::episode::EpisodeService;
use crate::services::whisper::WhisperService;
use crate::storage::database::Database;
use crate::storage::manager::DownloadManager;

#[derive(Clone)]
pub struct AppState {
    pub whisper_service: Arc<WhisperService>,
    pub rss_service: Arc<RSSFeedService>,
    pub podcast_service: Arc<PodcastService>,
    pub episode_service: Arc<EpisodeService>,
}

impl AppState {
    pub fn new(
        db: Database,
        base_download_path: String,
        whisper_url: String
    ) -> Self {
        let http_client = reqwest::Client::new();

        let download_manager = Arc::new(DownloadManager::new(
            PathBuf::from(base_download_path),
            http_client.clone()
        ));

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

        let episode_service = EpisodeService::new(
            db.episode_repository(),
            Arc::clone(&download_manager)
        );

        Self {
            whisper_service: Arc::new(whisper_service),
            rss_service: Arc::new(rss_service),
            podcast_service: Arc::new(podcast_service),
            episode_service: Arc::new(episode_service),
        }
    }
}