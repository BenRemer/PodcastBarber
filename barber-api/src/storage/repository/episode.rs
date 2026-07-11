use sqlx::SqlitePool;
use uuid::Uuid;
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

    pub async fn upsert(
        &self,
        episode: Episode
    ) -> Result<Episode, AppError> {
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
                id as "id!: Uuid",
                podcast_id as "podcast_id!: Uuid",
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
                AppError::InternalServerError("Failed to save episode to database".into())
            })?;

        Ok(row)
    }

    pub async fn get(
        &self,
        uid: &Uuid
    ) -> Result<Option<Episode>, AppError> {
        let episode = sqlx::query_as!(
            Episode,
            r#"
            SELECT
                id as "id!: Uuid",
                podcast_id as "podcast_id!: Uuid",
                guid as "guid!",
                title as "title!",
                audio_url as "audio_url!",
                local_file_path,
                state as "state!: EpisodeState"
            FROM episodes
            WHERE id = ?
            "#,
            uid
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB fetch_optional failed for episode {}: {}", uid, e);
                AppError::InternalServerError("Failed to fetch episode".into())
            })?;

        Ok(episode)
    }

    pub async fn get_by_podcast_id(
        &self,
        podcast_id: &Uuid
    ) -> Result<Vec<Episode>, AppError> {
        let episodes = sqlx::query_as!(
            Episode,
            r#"
            SELECT
                id as "id!: Uuid",
                podcast_id as "podcast_id!: Uuid",
                guid as "guid!",
                title as "title!",
                audio_url as "audio_url!",
                local_file_path,
                state as "state!: EpisodeState"
            FROM episodes
            WHERE podcast_id = ?
            -- Assuming you add a created_at or publish_date later, you would ORDER BY that here
            "#,
            podcast_id
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB fetch failed for podcast {} episodes: {}", podcast_id, e);
                AppError::InternalServerError("Failed to fetch episodes".into())
            })?;

        Ok(episodes)
    }

    pub async fn get_by_guid(
        &self,
        podcast_id: &Uuid,
        guid: &str
    ) -> Result<Option<Episode>, AppError> {
        let episode = sqlx::query_as!(
            Episode,
            r#"
            SELECT
                id as "id!: Uuid",
                podcast_id as "podcast_id!: Uuid",
                guid as "guid!",
                title as "title!",
                audio_url as "audio_url!",
                local_file_path,
                state as "state!: EpisodeState"
            FROM episodes
            WHERE podcast_id = ? AND guid = ?
            "#,
            podcast_id,
            guid
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB fetch_optional failed for episode {}: {}", guid, e);
                AppError::InternalServerError("Failed to fetch episode".into())
            })?;

        Ok(episode)
    }

    pub async fn delete(
        &self, id: &Uuid
    ) -> Result<bool, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM episodes
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB delete failed for episode {}: {}", id, e);
            AppError::InternalServerError("Failed to delete episode".into())
        })?;

        // Returns true if a record was actually deleted, false if the ID did not exist
        Ok(result.rows_affected() > 0)
    }
}