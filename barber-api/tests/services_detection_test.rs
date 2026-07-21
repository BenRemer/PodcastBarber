use crate::common::read_json_from_assets;
use barber_api::services::detection::{DetectionConfig, DetectionCore, Detector, generate_chunks};
use serde_json::Value;

mod common;

#[tokio::test]
pub async fn test_core() {
    let core = DetectionCore::new(DetectionConfig::default());

    let json: Value = read_json_from_assets("transcript.json").await;

    let chunks = generate_chunks(&json, 5.0).expect("Generating chunks failed");
    let segments = core.detect_ads(&chunks);

    for (index, segment) in segments.iter().enumerate() {
        if segment.is_ad {
            println!("found add");
            println!("before {:#?}", segments[index - 1]);
            println!("{:#?}", segment);
            println!("after {:#?}", segments.get(index + 1));
        }
    }

    assert!(segments.len() > 4);
}
