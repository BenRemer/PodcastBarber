use sqlx::SqlitePool;
use crate::error::AppError;
use crate::models::Podcast;

#[derive(Clone)]
pub struct PodcastRepository {
    pool: SqlitePool,
}

impl PodcastRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
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
}