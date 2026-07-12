use serde_json::Value;

pub struct DetectionCore {}

// split core into steps, manual, ai, etc? todo
impl DetectionCore {
    pub fn new() -> Self {
        Self {}
    }

    pub fn detect_ads(&self, json: Value) {
        println!("{:#?}", json);
        return;
    }
}
