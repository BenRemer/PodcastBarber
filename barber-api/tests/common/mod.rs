use std::sync::Arc;
use barber_api::services::rss_feed::RSSFeedService;
use barber_api::storage::download_manager::DownloadManager;
use tempfile::{tempdir, TempDir};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};
use crate::common;

pub struct TestContext {
    pub service: RSSFeedService,
    pub mock_server: MockServer,
    pub temp_dir: TempDir,
    pub download_manager: Arc<DownloadManager>,
}

impl TestContext {
    pub async fn setup() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let download_manager = Arc::new(DownloadManager::new(
            temp_dir.path().to_path_buf()
        ));
        let service = RSSFeedService::new(Arc::clone(&download_manager));
        Self {
            service,
            mock_server: MockServer::start().await,
            temp_dir,
            download_manager,
        }
    }

    pub fn output_path(&self) -> &str {
        self.temp_dir.path().to_str().unwrap()
    }

    pub async fn create_xml_feed_url(&self, asset_name: &str) -> String {
        let asset_path = common::get_asset_path(asset_name);
        let xml_bytes = tokio::fs::read(asset_path)
            .await
            .expect("Unable to read XML bytes");

        let realistic_response = ResponseTemplate::new(200)
            .insert_header("content-type", "application/rss+xml; charset=utf-8")
            .set_body_bytes(xml_bytes);

        Mock::given(method("GET"))
            .and(path("/feed.xml"))
            .respond_with(realistic_response)
            .mount(&self.mock_server)
            .await;

        format!("{}/feed.xml", self.mock_server.uri())
    }
}

pub fn get_asset_path(filename: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
        .join(filename)
}