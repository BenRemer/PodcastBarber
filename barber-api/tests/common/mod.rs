pub mod builder;
mod context;

pub use context::TestContext;
use serde::Serialize;
use serde::de::DeserializeOwned;
use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use tokio::fs;
pub fn get_asset_path(filename: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
        .join(filename)
}

pub async fn save_json_to_assets<T: Serialize>(filename: &str, data: &T) {
    let json_string =
        serde_json::to_string_pretty(data).expect("Failed to serialize data to string");

    let output_path = get_asset_path(filename);

    fs::write(&output_path, json_string)
        .await
        .expect(&format!("Failed to save JSON to disk at {:?}", output_path));
}

pub async fn read_json_from_assets<T: DeserializeOwned>(filename: &str) -> T {
    let input_path = get_asset_path(filename);

    let json_string = fs::read_to_string(&input_path).await.expect(&format!(
        "Failed to read JSON file from disk at {:?}",
        input_path
    ));

    serde_json::from_str(&json_string)
        .expect(&format!("Failed to parse JSON from {:?}", input_path))
}
