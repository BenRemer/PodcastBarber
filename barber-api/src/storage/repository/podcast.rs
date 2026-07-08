use sqlx::SqlitePool;
use uuid::Uuid;
use crate::error::AppError;
use crate::models::podcast::Podcast;

#[derive(Clone)]
pub struct PodcastRepository {
    pool: SqlitePool,
}

impl PodcastRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        podcast: Podcast,
    ) -> Result<Podcast, AppError> {
        let row = sqlx::query_as!(
        Podcast,
        r#"
        INSERT INTO podcasts (
            id,
            feed_url,
            title,
            image_url,
            description,
            author
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            image_url = excluded.image_url,
            description = excluded.description,
            author = excluded.author
        RETURNING
            id as "id!: uuid::Uuid",
            feed_url as "feed_url!",
            title as "title!",
            image_url,
            description,
            author
        "#,
        podcast.id,
        podcast.feed_url,
        podcast.title,
        podcast.image_url,
        podcast.description,
        podcast.author,
    )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB insert failed: {}", e);
                AppError::InternalServerError("Failed to insert podcast".into())
            })?;

        Ok(row)
    }

    pub async fn get_all(&self) -> Result<Vec<Podcast>, AppError> {
        let podcasts = sqlx::query_as!(
            Podcast,
            r#"
            SELECT
                id as "id!: uuid::Uuid",
                feed_url as "feed_url!",
                title as "title!",
                image_url,
                description,
                author
            FROM podcasts
            ORDER BY title
            "#
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB get_all failed: {}", e);
                AppError::InternalServerError("Failed to fetch podcast".into())
            })?;

        Ok(podcasts)
    }

    pub async fn get_podcast_by_id(&self, id: &Uuid) -> Result<Option<Podcast>, AppError> {
        let podcast = sqlx::query_as!(
            Podcast,
            r#"
            SELECT
                id as "id!: uuid::Uuid",
                feed_url as "feed_url!",
                title as "title!",
                image_url,
                description,
                author
            FROM podcasts
            WHERE id = ?
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB get_all failed: {}", e);
                AppError::InternalServerError("Failed to fetch podcast".into())
            })?;

        Ok(podcast)
    }

    pub async fn is_subscribed_feed(&self, feed_url: &str) -> Result<bool, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(1) FROM podcasts WHERE feed_url = ?",
            feed_url
        )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB exists check failed: {}", e);
                AppError::InternalServerError("Failed to check subscription status".into())
            })?;

        Ok(count > 0)
    }

    pub async fn is_subscribed_id(&self, id: &Uuid) -> Result<bool, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(1) FROM podcasts WHERE id = ?",
            id
        )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB exists check failed: {}", e);
                AppError::InternalServerError("Failed to check subscription status".into())
            })?;

        Ok(count > 0)
    }
}