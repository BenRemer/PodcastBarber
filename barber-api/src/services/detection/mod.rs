mod chunker;
mod core;
mod manual;
pub(crate) mod math;
mod segmenter;
mod service;
mod types;
mod worker;

pub use chunker::generate_chunks;
pub use core::{DetectionConfig, DetectionCore, Detector};
pub use service::DetectionService;
pub use types::*;
pub use worker::DetectionWorker;
