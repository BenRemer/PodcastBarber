use crate::{error::AppError, models::transcript::Transcript};

use serde_json::Value;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct TranscriptRepository {
    pool: SqlitePool,
}

#[derive(sqlx::FromRow)]
struct TranscriptRow {
    id: Uuid,
    episode_id: Uuid,
    data: String,
}

impl TranscriptRow {
    fn into_model(self) -> Result<Transcript, AppError> {
        let data = serde_json::from_str::<Value>(&self.data).map_err(|e| {
            tracing::error!("Invalid transcript JSON: {}", e);

            AppError::InternalServerError("Invalid transcript data".into())
        })?;

        Ok(Transcript {
            id: self.id,
            episode_id: self.episode_id,
            data,
        })
    }
}

impl TranscriptRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, transcript: Transcript) -> Result<Transcript, AppError> {
        let data = serde_json::to_string(&transcript.data).map_err(|e| {
            tracing::error!("Failed serializing transcript: {}", e);

            AppError::InternalServerError("Failed to serialize transcript".into())
        })?;

        let row = sqlx::query_as::<_, TranscriptRow>(
            r#"
                INSERT INTO transcripts (
                    id,
                    episode_id,
                    data
                )
                VALUES (?, ?, ?)

                ON CONFLICT(episode_id)
                DO UPDATE SET
                    data = excluded.data

                RETURNING
                    id,
                    episode_id,
                    data
                "#,
        )
        .bind(transcript.id)
        .bind(transcript.episode_id)
        .bind(data)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed saving transcript: {}", e);

            AppError::InternalServerError("Failed saving transcript".into())
        })?;

        row.into_model()
    }

    pub async fn get_by_episode_id(
        &self,
        episode_id: &Uuid,
    ) -> Result<Option<Transcript>, AppError> {
        let row = sqlx::query_as::<_, TranscriptRow>(
            r#"
                SELECT
                    id,
                    episode_id,
                    data
                FROM transcripts
                WHERE episode_id = ?
                "#,
        )
        .bind(episode_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed fetching transcript: {}", e);

            AppError::InternalServerError("Failed fetching transcript".into())
        })?;

        match row {
            Some(row) => Ok(Some(row.into_model()?)),
            None => Ok(None),
        }
    }
}
