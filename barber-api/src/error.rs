use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    InvalidInput(String),
    SidecarUnavailable,
    InternalServerError(String),
    NotFound,
}

// Tell Axum how to convert these errors into HTTP responses
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InvalidInput(err) => {
                tracing::error!("Invalid input: {}", err);
                (StatusCode::BAD_REQUEST, "Invalid input")
            },
            AppError::SidecarUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "The Whisper sidecar is not responding",
            ),
            AppError::InternalServerError(err) => {
                tracing::error!("Internal error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected internal error occurred",
                )
            },
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}