use crate::common::context::TestContext;
use crate::common::mocks;

mod common;

#[tokio::test]
async fn test_list_episodes() {
    let ctx = TestContext::builder().build().await;
    let feed_url = mocks::rss::create_feed(&ctx.mock_server, "feed.xml").await;

    let new_ten = ctx
        .rss_service
        .list_episodes(&feed_url, Some(10))
        .await
        .expect("Service failed to list episodes");

    assert_eq!(new_ten.len(), 10);
}
