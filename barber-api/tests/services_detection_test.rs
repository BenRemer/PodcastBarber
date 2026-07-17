use crate::common::read_json_from_assets;
use barber_api::services::detection::DetectionCore;
use serde_json::Value;

mod common;

#[tokio::test]
pub async fn test_core() {
    // let ctx = TestContext::builder().with_background_workers().build().await;
    let core = DetectionCore::new();

    let json: Value = read_json_from_assets("transcript.json").await;

    let segments = core.detect_ads(&json).expect("expected result");

    for (index, segment) in segments.iter().enumerate() {
        if segment.is_ad {
            println!("found add");
            println!("before {:#?}", segments[index - 1]);
            println!("{:#?}", segment);
            println!("after {:#?}", segments.get(index + 1));
        }
    }
}
