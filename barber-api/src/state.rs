use std::sync::Arc;
use crate::services::rss::RSSFeedService;
use crate::services::podcast::PodcastService;
use crate::services::episode::EpisodeService;
use crate::services::whisper::WhisperService;

#[derive(Clone)]
pub struct AppState {
    pub whisper_service: Arc<WhisperService>,
    pub rss_service: Arc<RSSFeedService>,
    pub podcast_service: Arc<PodcastService>,
    pub episode_service: Arc<EpisodeService>,
}

impl AppState {
    pub fn new(
        whisper_service: Arc<WhisperService>,
        rss_service: Arc<RSSFeedService>,
        podcast_service: Arc<PodcastService>,
        episode_service: Arc<EpisodeService>,
    ) -> Self {
        Self {
            whisper_service,
            rss_service,
            podcast_service,
            episode_service,
        }
    }
}