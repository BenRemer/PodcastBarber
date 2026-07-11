use crate::common::TestContext;

mod common;

#[tokio::test]
async fn test_list_episodes() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;

    let new_ten = ctx.rss_service
        .list_episodes(&feed_url, Some(10))
        .await
        .expect("Service failed to list episodes");

    assert_eq!(new_ten.len(), 10);
}
