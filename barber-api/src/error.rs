use aide::OperationOutput;
use aide::generate::GenContext;
use aide::openapi::Response as OpenApiResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, Clone)]
pub enum AppError {
    BadRequest(String),
    SidecarUnavailable,
    InternalServerError(String),
    NotFound,
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        tracing::error!("Reqwest HTTP error: {:?}", err);
        AppError::InternalServerError(format!("Network request failed: {}", err))
    }
}

// Tell Axum how to convert these errors into HTTP responses
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(err) => {
                tracing::error!("Invalid input: {}", err);
                (StatusCode::BAD_REQUEST, "Invalid input")
            }
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
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// Tell aide how to convert to http errors
impl OperationOutput for AppError {
    type Inner = Self;

    fn inferred_responses(
        _ctx: &mut GenContext,
        _operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, OpenApiResponse)> {
        vec![
            (
                Some(400),
                OpenApiResponse {
                    description: "Bad Request".into(),
                    ..Default::default()
                },
            ),
            (
                Some(404),
                OpenApiResponse {
                    description: "Not Found".into(),
                    ..Default::default()
                },
            ),
            (
                Some(500),
                OpenApiResponse {
                    description: "Internal Server Error".into(),
                    ..Default::default()
                },
            ),
            (
                Some(503),
                OpenApiResponse {
                    description: "Service Unavailable".into(),
                    ..Default::default()
                },
            ),
        ]
    }
}
