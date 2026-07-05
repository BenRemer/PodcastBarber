use tokio::fs;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use crate::common::TestContext;

mod common;

#[tokio::test]
async fn test_get_latest_mp3() {
    let ctx = TestContext::setup().await;
    let fake_audio_path = "/episodes/test_audio.mp3";
    let fake_audio_url = format!("{}{}", ctx.mock_server.uri(), fake_audio_path);
    let fake_audio_bytes = b"hello this is a fake mp3 stream";

    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            r#"<rss><channel><item><title>Test Episode</title><enclosure url="{}" type="audio/mpeg"/></item></channel></rss>"#,
            fake_audio_url
        )))
        .mount(&ctx.mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path(fake_audio_path))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(fake_audio_bytes.to_vec()))
        .mount(&ctx.mock_server)
        .await;

    let feed_url = ctx.create_xml_feed_url("feed.xml").await;
    let saved_file_path = ctx.feed_service
        .download_latest_episode(&feed_url)
        .await
        .expect("Service failed to download episode");

    let expected_hash = barber_api::utils::generate_deterministic_hash(&fake_audio_url);
    assert_eq!(saved_file_path.to_str().unwrap().contains(&expected_hash), true);
    let file_contents = fs::read(&saved_file_path).await.unwrap();
    assert_eq!(file_contents, fake_audio_bytes);
}

#[tokio::test]
async fn test_list_episodes() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;

    let new_ten = ctx.feed_service
        .list_episodes(&feed_url, 10)
        .await
        .expect("Service failed to list episodes");

    assert_eq!(new_ten.len(), 10);
}

#[tokio::test]
async fn test_save_episode() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;

    let episode_list = ctx.feed_service
        .list_episodes(&feed_url, 1)
        .await
        .expect("Service failed to list episodes");
    assert_eq!(episode_list.len(), 1);

    let newest_episode = episode_list.first().expect("Empty episode");
    let id = &newest_episode.guid;

    let episode_path = ctx.feed_service
        .download_episode(&feed_url, &id)
        .await
        .expect("Service failed to download episode");
    assert!(episode_path.exists());
    assert_eq!(episode_path.to_str().unwrap().contains(id.as_str()), true);
}

#[tokio::test]
async fn test_save_nonexistent_episode() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;

    let failure = ctx.feed_service
        .download_episode(&feed_url, "12345")
        .await;

    assert!(failure.is_err());
}