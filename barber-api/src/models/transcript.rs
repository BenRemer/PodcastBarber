use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Transcript {
    pub id: Uuid,
    pub episode_id: Uuid,
    pub data: Value,
}
