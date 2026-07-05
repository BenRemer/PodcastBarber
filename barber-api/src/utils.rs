use sha2::{Sha256, Digest};

/// Generates a deterministic, filesystem-safe hex string from an input string.
/// This is used to create fallback GUIDs for podcast episodes that lack one.
pub fn generate_deterministic_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}