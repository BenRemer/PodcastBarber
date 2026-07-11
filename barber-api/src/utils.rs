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
