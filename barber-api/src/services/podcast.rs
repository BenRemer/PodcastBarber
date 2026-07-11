use crate::error::AppError;
use crate::models::podcast::Podcast;
use crate::storage::repository::podcast::PodcastRepository;
use uuid::Uuid;

#[derive(Clone)]
pub struct PodcastService {
    podcast_repository: PodcastRepository,
}

impl PodcastService {
    pub fn new(podcast_repository: PodcastRepository) -> Self {
        Self { podcast_repository }
    }

    pub async fn list_podcasts(&self) -> Result<Vec<Podcast>, AppError> {
        self.podcast_repository.get_all().await
    }

    pub async fn subscribe_podcast(&self, podcast: Podcast) -> Result<Podcast, AppError> {
        self.podcast_repository.upsert(podcast).await
    }

    pub async fn is_subscribed_by_url(&self, feed_url: &str) -> Result<bool, AppError> {
        self.podcast_repository.is_subscribed_feed(feed_url).await
    }

    pub async fn is_subscribed_by_id(&self, id: &Uuid) -> Result<bool, AppError> {
        self.podcast_repository.is_subscribed_id(id).await
    }

    pub async fn get_podcast_by_id(&self, id: &Uuid) -> Result<Option<Podcast>, AppError> {
        self.podcast_repository.get_podcast_by_id(id).await
    }
}
