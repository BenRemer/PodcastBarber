use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Generates a deterministic, filesystem-safe hex string from an input string.
/// This is used to create fallback GUIDs for podcast episodes that lack one.
pub fn generate_deterministic_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_podcast_uuid(feed_url: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, feed_url.as_bytes())
}

pub fn generate_episode_uuid(podcast_id: Uuid, guid: &str) -> Uuid {
    Uuid::new_v5(&podcast_id, guid.as_bytes())
}

pub fn get_content_type(file_bytes: &Vec<u8>) -> String {
    let content_type = match infer::get(&file_bytes) {
        Some(kind) => kind.mime_type().to_string(),
        None => "application/octet-stream".to_string(),
    };
    content_type
}
