use crate::common::TestContext;
use barber_api::models::podcast::Podcast;

mod common;

#[tokio::test]
async fn test_subscribe_new_podcast() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;
    let metadata = ctx
        .rss_service
        .fetch_podcast_metadata(&feed_url)
        .await
        .expect("Fetch failed");
    let podcast = Podcast::from(metadata);

    let podcast = ctx
        .podcast_service
        .subscribe_podcast(podcast)
        .await
        .expect("Subscribe failed");
    let metadata = ctx
        .rss_service
        .fetch_podcast_metadata(&feed_url)
        .await
        .expect("Fetch failed");
    let expected: Podcast = metadata.into();

    assert_eq!(expected, podcast);
}

#[tokio::test]
async fn test_podcast_is_subscribed_url() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;
    let metadata = ctx
        .rss_service
        .fetch_podcast_metadata(&feed_url)
        .await
        .expect("Fetch failed");
    let podcast = Podcast::from(metadata);

    let exists_before = ctx
        .podcast_service
        .is_subscribed_by_url(&feed_url)
        .await
        .expect("Failed to execute exists check");

    assert_eq!(exists_before, false, "Database should be empty initially");

    ctx.podcast_service
        .subscribe_podcast(podcast)
        .await
        .expect("Failed to insert fake podcast");

    let exists_after = ctx
        .podcast_service
        .is_subscribed_by_url(&feed_url)
        .await
        .expect("Failed to execute exists check");

    assert_eq!(
        exists_after, true,
        "Database should confirm the podcast exists"
    );
}

#[tokio::test]
async fn test_podcast_is_subscribed_id() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;
    let metadata = ctx
        .rss_service
        .fetch_podcast_metadata(&feed_url)
        .await
        .expect("Fetch failed");
    let podcast = Podcast::from(metadata);

    let exists_before = ctx
        .podcast_service
        .is_subscribed_by_id(&podcast.id)
        .await
        .expect("Failed to execute exists check");

    assert_eq!(exists_before, false, "Database should be empty initially");

    ctx.podcast_service
        .subscribe_podcast(podcast.clone())
        .await
        .expect("Failed to insert fake podcast");

    let exists_after = ctx
        .podcast_service
        .is_subscribed_by_id(&podcast.id)
        .await
        .expect("Failed to execute exists check");

    assert_eq!(
        exists_after, true,
        "Database should confirm the podcast exists"
    );
}

#[tokio::test]
async fn test_subscribe_podcast_idempotency_prevents_duplicates() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;
    let metadata = ctx
        .rss_service
        .fetch_podcast_metadata(&feed_url)
        .await
        .expect("Fetch failed");
    let podcast = Podcast::from(metadata);

    let exists_before = ctx
        .podcast_service
        .is_subscribed_by_url(&feed_url)
        .await
        .expect("Failed to execute exists check");
    assert_eq!(exists_before, false, "Database should be empty initially");

    let first_result = ctx
        .podcast_service
        .subscribe_podcast(podcast.clone())
        .await
        .expect("First subscription failed");

    let second_result = ctx
        .podcast_service
        .subscribe_podcast(podcast.clone())
        .await
        .expect("Second subscription failed");

    assert_eq!(
        first_result.id, second_result.id,
        "The returned UUIDs must match perfectly due to v5 hashing"
    );

    // The database must only contain exactly ONE row for this URL.
    let db_row_count: i32 =
        sqlx::query_scalar!("SELECT COUNT(*) FROM podcasts WHERE feed_url = ?", feed_url)
            .fetch_one(&ctx.pool)
            .await
            .expect("Failed to query database for podcast count") as i32;

    assert_eq!(
        db_row_count, 1,
        "Database should contain exactly 1 row, proving the UPSERT worked"
    );
}
