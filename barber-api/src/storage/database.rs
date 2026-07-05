use std::str::FromStr;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use crate::storage::repository::PodcastRepository;

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::from_str(url)?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    pub fn podcast_repository(&self) -> PodcastRepository {
        PodcastRepository::new(self.pool.clone())
    }
}