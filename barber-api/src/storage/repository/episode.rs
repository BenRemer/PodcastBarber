use std::path::PathBuf;
use crate::error::AppError;
use crate::models::episode::{Episode, EpisodeState};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct EpisodeRepository {
    pool: SqlitePool,
}

#[derive(sqlx::FromRow)]
struct EpisodeRow {
    id: Uuid,
    podcast_id: Uuid,
    guid: String,
    title: String,
    audio_url: String,
    local_file_path: Option<String>,
    state: EpisodeState,
}

impl From<EpisodeRow> for Episode {
    fn from(row: EpisodeRow) -> Self {
        Self {
            id: row.id,
            podcast_id: row.podcast_id,
            guid: row.guid.clone(),
            title: row.title.clone(),
            audio_url: row.audio_url.clone(),
            local_file_path: row.local_file_path.map(PathBuf::from),
            state: row.state,
        }
    }
}

impl From<&Episode> for EpisodeRow {
    fn from(ep: &Episode) -> Self {
        Self {
            id: ep.id,
            podcast_id: ep.podcast_id,
            guid: ep.guid.clone(),
            title: ep.title.clone(),
            audio_url: ep.audio_url.clone(),
            local_file_path: ep
                .local_file_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            state: ep.state,
        }
    }
}

impl EpisodeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, episode: Episode) -> Result<Episode, AppError> {
        let db_episode = EpisodeRow::from(&episode);

        let row = sqlx::query_as!(
            EpisodeRow,
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
            db_episode.id,
            db_episode.podcast_id,
            db_episode.guid,
            db_episode.title,
            db_episode.audio_url,
            db_episode.local_file_path,
            db_episode.state as EpisodeState
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB upsert failed for episode: {}", e);
            AppError::InternalServerError("Failed to save episode to database".into())
        })?;

        Ok(Episode::from(row))
    }

    pub async fn get(&self, uid: &Uuid) -> Result<Option<Episode>, AppError> {
        let row = sqlx::query_as!(
            EpisodeRow,
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

        Ok(row.map(Episode::from))
    }

    pub async fn get_by_podcast_id(&self, podcast_id: &Uuid) -> Result<Vec<Episode>, AppError> {
        let row = sqlx::query_as!(
            EpisodeRow,
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
            "#,
            podcast_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB fetch failed for podcast {} episodes: {}", podcast_id, e);
            AppError::InternalServerError("Failed to fetch episodes".into())
        })?;

        Ok(row.into_iter().map(Episode::from).collect())
    }

    pub async fn get_by_guid(
        &self,
        podcast_id: &Uuid,
        guid: &str,
    ) -> Result<Option<Episode>, AppError> {
        let row = sqlx::query_as!(
            EpisodeRow,
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

        Ok(row.map(Episode::from))
    }

    pub async fn delete(&self, id: &Uuid) -> Result<bool, AppError> {
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
