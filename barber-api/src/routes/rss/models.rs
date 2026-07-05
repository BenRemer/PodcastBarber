use rss::Item;
use serde::Serialize;

#[derive(Serialize)]
pub struct ItemsResponse {
    pub items: Vec<Item>,
    pub total: usize,
}