use crate::storage::repository::detection::DetectionRepository;
use crate::storage::repository::episode::EpisodeRepository;
use crate::storage::repository::podcast::PodcastRepository;
use crate::storage::repository::transcript::TranscriptRepository;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);

        let pool = SqlitePoolOptions::new().connect_with(options).await?;

        Ok(Self { pool })
    }

    pub fn podcast_repository(&self) -> PodcastRepository {
        PodcastRepository::new(self.pool.clone())
    }

    pub fn episode_repository(&self) -> EpisodeRepository {
        EpisodeRepository::new(self.pool.clone())
    }

    pub fn transcript_repository(&self) -> TranscriptRepository {
        TranscriptRepository::new(self.pool.clone())
    }

    pub fn detection_repository(&self) -> DetectionRepository {
        DetectionRepository::new(self.pool.clone())
    }
}
