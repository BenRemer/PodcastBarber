use barber_api::models::Podcast;
use crate::common::TestContext;


mod common;

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

    let episode_path = ctx.podcast_service
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

    let failure = ctx.podcast_service
        .download_episode(&feed_url, "12345")
        .await;

    assert!(failure.is_err());
}

#[tokio::test]
async fn test_subscribe_new_podcast() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;

    let podcast = ctx.podcast_service.subscribe_podcast(&feed_url).await.expect("Subscribe failed");
    let metadata = ctx.feed_service.fetch_podcast_metadata(&feed_url).await.expect("Fetch failed");
    let expected: Podcast = metadata.into();

    assert_eq!(expected, podcast);
}

#[tokio::test]
async fn test_podcast_is_subscribed() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;

    let exists_before = ctx.podcast_service
        .is_subscribed(&feed_url)
        .await
        .expect("Failed to execute exists check");

    assert_eq!(exists_before, false, "Database should be empty initially");

    ctx.podcast_service
        .subscribe_podcast(&feed_url)
        .await
        .expect("Failed to insert fake podcast");

    let exists_after = ctx.podcast_service
        .is_subscribed(&feed_url)
        .await
        .expect("Failed to execute exists check");

    assert_eq!(exists_after, true, "Database should confirm the podcast exists");
}

#[tokio::test]
async fn test_subscribe_podcast_idempotency_prevents_duplicates() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;

    let exists_before = ctx.podcast_service
        .is_subscribed(&feed_url)
        .await
        .expect("Failed to execute exists check");
    assert_eq!(exists_before, false, "Database should be empty initially");

    let first_result = ctx.podcast_service
        .subscribe_podcast(&feed_url)
        .await
        .expect("First subscription failed");

    let second_result = ctx.podcast_service
        .subscribe_podcast(&feed_url)
        .await
        .expect("Second subscription failed");

    assert_eq!(
        first_result.id, second_result.id,
        "The returned UUIDs must match perfectly due to v5 hashing"
    );

    // The database must only contain exactly ONE row for this URL.
    let db_row_count: i32 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM podcasts WHERE feed_url = ?",
        feed_url
    )
        .fetch_one(&ctx.pool)
        .await
        .expect("Failed to query database for podcast count") as i32;

    assert_eq!(
        db_row_count, 1,
        "Database should contain exactly 1 row, proving the UPSERT worked"
    );
}