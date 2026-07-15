use crate::common::{TestContext, read_json_from_assets};
use barber_api::services::detection::DetectionCore;
use serde_json::Value;

mod common;

#[tokio::test]
pub async fn test_core() {
    // let ctx = TestContext::builder().with_background_workers().build().await;
    let core = DetectionCore::new();

    let json: Value = read_json_from_assets("bologna-speech-english.json").await;

    core.detect_ads(json);
}
