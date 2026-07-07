use sqlx::SqlitePool;
use crate::error::AppError;
use crate::models::episode::{Episode, EpisodeState};

#[derive(Clone)]
pub struct EpisodeRepository {
    pool: SqlitePool,
}

impl EpisodeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, episode: Episode) -> Result<Episode, AppError> {
        let row = sqlx::query_as!(
            Episode,
            r#"
            INSERT INTO episodes (
                id, podcast_id, guid, title, audio_url, local_file_path, state
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(podcast_id, guid) DO UPDATE SET
                title = excluded.title,
                audio_url = excluded.audio_url,
                local_file_path = excluded.local_file_path,
                state = excluded.state
            RETURNING
                id as "id!: uuid::Uuid",
                podcast_id as "podcast_id!: uuid::Uuid",
                guid as "guid!",
                title as "title!",
                audio_url as "audio_url!",
                local_file_path,
                state as "state!: EpisodeState"
            "#,
            episode.id,
            episode.podcast_id,
            episode.guid,
            episode.title,
            episode.audio_url,
            episode.local_file_path,
            episode.state as EpisodeState,
        )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB upsert failed for episode: {}", e);
                AppError::InternalServerError("Failed to save episode".into())
            })?;

        Ok(row)
    }
}