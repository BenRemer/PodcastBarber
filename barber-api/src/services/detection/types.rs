use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct DetectionJob {
    pub tracking_id: Uuid,
}

pub struct DetectionResult {
    pub tracking_id: Uuid,
}
