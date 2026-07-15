use crate::services::detection::DetectionService;
use crate::services::episode::EpisodeService;
use crate::services::podcast::PodcastService;
use crate::services::rss::RSSFeedService;
use crate::services::transcribe::TranscribeService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub whisper_service: Arc<TranscribeService>,
    pub rss_service: Arc<RSSFeedService>,
    pub podcast_service: Arc<PodcastService>,
    pub episode_service: Arc<EpisodeService>,
    pub detection_service: Arc<DetectionService>,
}

impl AppState {
    pub fn new(
        whisper_service: Arc<TranscribeService>,
        rss_service: Arc<RSSFeedService>,
        podcast_service: Arc<PodcastService>,
        episode_service: Arc<EpisodeService>,
        detection_service: Arc<DetectionService>,
    ) -> Self {
        Self {
            whisper_service,
            rss_service,
            podcast_service,
            episode_service,
            detection_service,
        }
    }
}
