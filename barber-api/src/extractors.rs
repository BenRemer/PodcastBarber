use crate::error::AppError;
use axum::extract::{FromRequest, Multipart, Request};
use bytes::Bytes;

pub struct AudioUpload {
    pub file_name: String,
    pub content_type: String,
    pub data: Bytes,
}

impl<S> FromRequest<S> for AudioUpload
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        tracing::info!("Receiving multipart audio upload...");

        let mut multipart = Multipart::from_request(req, state).await.map_err(|e| {
            tracing::error!("Failed to parse multipart: {}", e);
            AppError::InternalServerError("Failed to read upload stream".into())
        })?;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| AppError::InternalServerError("Failed to read stream chunk".into()))?
        {
            if field.name() == Some("file") {
                let file_name = field.file_name().unwrap_or("upload.audio").to_string();
                let content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::InternalServerError("Failed to extract bytes".into()))?;

                return Ok(AudioUpload {
                    file_name,
                    content_type,
                    data,
                });
            }
        }

        Err(AppError::InternalServerError(
            "No 'file' field found in form".into(),
        ))
    }
}
