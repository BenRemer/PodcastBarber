use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::common;

pub async fn create_feed(server: &wiremock::MockServer, asset_name: &str) -> String {
    let asset_path = common::get_asset_path(asset_name);

    let xml_bytes = tokio::fs::read(asset_path)
        .await
        .expect("Unable to read XML fixture");

    let unique_path = format!("/feed-{}.xml", Uuid::new_v4());

    Mock::given(method("GET"))
        .and(path(unique_path.clone()))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/rss+xml; charset=utf-8")
                .set_body_bytes(xml_bytes),
        )
        .mount(server)
        .await;

    format!("{}{}", server.uri(), unique_path)
}
