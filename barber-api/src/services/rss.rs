use reqwest::Client;
use rss::{Channel, Item};
use crate::error::AppError;
use crate::models::api::EpisodeItem;
use crate::models::podcast::PodcastMetadata;

#[derive(Clone)]
pub struct RSSFeedService {
    client: Client,
}

impl RSSFeedService {
    pub fn new(client: Client) -> Self {
        Self {
            client,
        }
    }

    pub async fn fetch_podcast_metadata(&self, feed_url: &str) -> Result<PodcastMetadata, AppError> {
        tracing::info!("Fetching feed {}", feed_url);
        let channel = self.construct_rss_channel(feed_url).await?;
        Ok(PodcastMetadata::from_channel(&channel, feed_url.to_string()))
    }

    pub(crate) fn extract_metadata(item: &Item) -> Result<EpisodeItem, AppError> {
        let title = item.title()
            .unwrap_or("Unknown Episode")
            .to_string();

        let audio_url = item.enclosure()
            .map(|enc| enc.url().to_string())
            .ok_or_else(|| {
                tracing::warn!("Episode '{}' has no audio enclosure", title);
                AppError::NotFound
            })?;

        let publish_date = item.pub_date().map(|date| date.to_string());

        let guid = item.guid()
            .map(|id| id.value().to_string())
            .unwrap_or_else(|| {
                let input_to_hash = item.enclosure()
                    .map(|e| e.url())
                    .or_else(|| item.link())
                    .or_else(|| item.title());

                match input_to_hash {
                    Some(input) => crate::utils::generate_deterministic_hash(input),
                    None => uuid::Uuid::new_v4().to_string(),
                }
            });

        Ok(EpisodeItem {
            guid,
            title,
            audio_url,
            publish_date
        })
    }

    pub(crate) fn parse_rss_channel(xml_bytes: &[u8]) -> Result<Channel, AppError> {
        Channel::read_from(xml_bytes)
            .map_err(|e| {
                println!("!!! RSS PARSING CRASHED BECAUSE: {:#?} !!!", e);
                tracing::error!("Failed to parse RSS: {}", e);
                AppError::InternalServerError("Invalid RSS format".into())
            })
    }

    async fn fetch_rss_bytes(&self, feed_url: &str) -> Result<bytes::Bytes, AppError> {
        self.client.get(feed_url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch RSS: {}", e);
                AppError::InternalServerError("Failed to reach feed URL".into())
            })?
            .bytes()
            .await
            .map_err(|_| AppError::InternalServerError("Failed to read XML payload".into()))
    }

    pub async fn construct_rss_channel(&self, feed_url: &str) -> Result<Channel, AppError> {
        let bytes = self.fetch_rss_bytes(feed_url).await?;
        Self::parse_rss_channel(&bytes)
    }

    pub async fn list_episodes(
        &self, feed_url: &str, limit: Option<usize>
    ) -> Result<Vec<EpisodeItem>, AppError> {
        let size = limit.unwrap_or(usize::MAX);
        tracing::info!("Listing up to {} episodes of {}", size, feed_url);

        let channel = self.construct_rss_channel(feed_url).await?; // todo pass in

        let items: Vec<EpisodeItem> = channel.items
            .iter()
            .filter_map(|item| Self::extract_metadata(item).ok())
            .take(size)
            .collect();

        Ok(items)
    }

    pub async fn get_episode_metadata(
        &self, channel: &Channel, feed_url: &str, guid: &str
    ) -> Result<EpisodeItem, AppError> {
        tracing::info!("Fetching episode {} from {}", guid, feed_url);

        channel.items()
            .iter()
            .find_map(|item| {
                match Self::extract_metadata(item) {
                    Ok(meta) if meta.guid == guid => Some(meta),
                    _ => None,
                }
            })
            .ok_or_else(|| {
                tracing::warn!("Episode with GUID '{}' not found in RSS feed", guid);
                AppError::NotFound
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rss::{ItemBuilder, EnclosureBuilder, GuidBuilder};

    #[test]
    fn test_extract_metadata_happy_path() {
        let enclosure = EnclosureBuilder::default()
            .url("https://example.com/audio.mp3".to_string())
            .build();

        let guid = GuidBuilder::default()
            .value("real-guid-123".to_string())
            .build();

        let item = ItemBuilder::default()
            .title(Some("Test Episode".to_string()))
            .enclosure(Some(enclosure))
            .guid(Some(guid))
            .pub_date(Some("Mon, 01 Jan 2026 00:00:00 GMT".to_string()))
            .build();

        let result = RSSFeedService::extract_metadata(&item).expect("Should succeed");

        assert_eq!(result.title, "Test Episode");
        assert_eq!(result.audio_url, "https://example.com/audio.mp3");
        assert_eq!(result.guid, "real-guid-123");
        assert_eq!(result.publish_date, Some("Mon, 01 Jan 2026 00:00:00 GMT".to_string()));
    }

    #[test]
    fn test_extract_metadata_missing_guid_uses_fallback() {
        let enclosure = EnclosureBuilder::default()
            .url("https://example.com/audio.mp3".to_string())
            .build();

        // Notice we are intentionally omitting the GUID here
        let item = ItemBuilder::default()
            .title(Some("No GUID Episode".to_string()))
            .enclosure(Some(enclosure))
            .build();

        let result = RSSFeedService::extract_metadata(&item).expect("Should succeed");

        let expected_hash = crate::utils::generate_deterministic_hash("https://example.com/audio.mp3");

        assert_eq!(result.guid, expected_hash);
    }

    #[test]
    fn test_extract_metadata_missing_enclosure_returns_error() {
        // Item with a title, but absolutely no enclosure (audio file)
        let item = ItemBuilder::default()
            .title(Some("Broken Episode".to_string()))
            .build();

        let result = RSSFeedService::extract_metadata(&item);

        // Assert that it correctly caught the error and threw AppError::NotFound
        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[test]
    fn test_parse_rss_channel_success() {
        // A tiny, perfectly valid XML payload
        let raw_xml = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <rss version="2.0">
                <channel>
                    <title>My Awesome Podcast</title>
                    <link>https://example.com</link>
                    <description>A podcast about testing.</description>
                </channel>
            </rss>
        "#;

        let channel = RSSFeedService::parse_rss_channel(raw_xml.as_bytes())
            .expect("Should parse valid XML");

        assert_eq!(channel.title(), "My Awesome Podcast");
    }

    #[test]
    fn test_parse_rss_channel_invalid_xml_returns_error() {
        // Missing the closing </channel> and </rss> tags!
        let broken_xml = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <rss version="2.0">
                <channel>
                    <title>Broken Podcast</title>
        "#;

        let result = RSSFeedService::parse_rss_channel(broken_xml.as_bytes());

        assert!(matches!(result, Err(AppError::InternalServerError(_))));
    }
}