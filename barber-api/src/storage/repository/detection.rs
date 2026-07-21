use crate::error::AppError;
use crate::services::detection::{Detection, ProcessedSegment};
use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

#[async_trait]
pub trait DetectionStore: Send + Sync {
    async fn upsert(&self, detection: Detection) -> Result<(), AppError>;
    async fn get_detection_by_episode(
        &self,
        episode_id: &Uuid,
    ) -> Result<Option<Detection>, AppError>;
}

#[async_trait]
impl DetectionStore for DetectionRepository {
    async fn upsert(&self, detection: Detection) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            tracing::error!("Failed starting transaction: {}", e);
            AppError::InternalServerError("Failed starting transaction".into())
        })?;

        for segment in detection.segments {
            let row = DetectionSegmentRow {
                id: Uuid::new_v4(),
                episode_id: detection.episode_id,
                start_time: segment.start_time,
                end_time: segment.end_time,
                text: segment.text,
                ad_score: segment.ad_score,
                is_ad: segment.is_ad,
            };

            self.upsert_row(&mut tx, row).await?;
        }

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed committing detection: {}", e);
            AppError::InternalServerError("Failed committing detection".into())
        })?;

        Ok(())
    }

    async fn get_detection_by_episode(
        &self,
        episode_id: &Uuid,
    ) -> Result<Option<Detection>, AppError> {
        let rows = sqlx::query_as::<_, DetectionSegmentRow>(
            r#"
                SELECT
                id,
                episode_id,
                start_time,
                end_time,
                text,
                ad_score,
                is_ad
                FROM detection_segments
                WHERE episode_id = ?
                ORDER BY start_time
            "#,
        )
        .bind(episode_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed getting detection segment: {}", e);
            AppError::InternalServerError("Failed getting detection segment".into())
        })?;

        if rows.is_empty() {
            return Ok(None);
        };

        let segments = rows
            .into_iter()
            .map(DetectionSegmentRow::into_segment)
            .collect();

        Ok(Some(Detection {
            episode_id: *episode_id,
            segments,
        }))
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DetectionSegmentRow {
    pub id: Uuid,
    pub episode_id: Uuid,
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
    pub ad_score: i32,
    pub is_ad: bool,
}

impl DetectionSegmentRow {
    fn into_segment(self) -> ProcessedSegment {
        ProcessedSegment {
            start_time: self.start_time,
            end_time: self.end_time,
            text: self.text,
            ad_score: self.ad_score,
            is_ad: self.is_ad,
        }
    }
}

#[derive(Clone)]
pub struct DetectionRepository {
    pool: SqlitePool,
}

impl DetectionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn upsert_row(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        row: DetectionSegmentRow,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO detection_segments (
                id,
                episode_id,
                start_time,
                end_time,
                text,
                ad_score,
                is_ad
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)

            ON CONFLICT(episode_id, start_time, end_time)
            DO UPDATE SET
                text = excluded.text,
                ad_score = excluded.ad_score,
                is_ad = excluded.is_ad
            "#,
            row.id,
            row.episode_id,
            row.start_time,
            row.end_time,
            row.text,
            row.ad_score,
            row.is_ad,
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed upserting detection segment: {}", e);
            AppError::InternalServerError("Failed saving detection segment".into())
        })?;

        Ok(())
    }
}
