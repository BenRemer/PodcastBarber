use rss::Channel;
use serde::Serialize;
use uuid::Uuid;
use crate::utils::generate_podcast_uuid;

pub struct PodcastMetadata {
    pub title: String,
    pub feed_url: String,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
}

impl PodcastMetadata {
    pub fn from_channel(channel: &Channel, feed_url: String) -> Self {
        Self {
            title: channel.title().to_string(),
            feed_url,
            image_url: channel.image().map(|i| i.url().to_string()),
            description: Some(channel.description().to_string())
                .filter(|s| !s.is_empty()),
            author: channel.itunes_ext()
                .and_then(|itunes| itunes.author().map(str::to_string)),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Podcast {
    pub id: Uuid,
    pub title: String,
    pub feed_url: String,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    // pub subscribed_at: DateTime<Utc>
}

impl From<PodcastMetadata> for Podcast {
    fn from(metadata: PodcastMetadata) -> Self {
        let deterministic_id = generate_podcast_uuid(&metadata.feed_url);
        Self {
            id: deterministic_id,
            title: metadata.title,
            feed_url: metadata.feed_url,
            image_url: metadata.image_url,
            description: metadata.description,
            author: metadata.author,
        }
    }
}