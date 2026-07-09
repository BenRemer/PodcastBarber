// Defaults to use in the app if .env is not set
pub const DATABASE_URL: &str = "sqlite://podcast.db";
pub const DEFAULT_WHISPER_URL: &str = "http://whisper_sidecar:8000/v1";
pub const BASE_DOWNLOAD_PATH: &str = "./downloads";
pub const DEFAULT_BUFFER_QUEUE: usize = 100;