use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard API error response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, String>>,
}

impl ApiErrorResponse {
    pub fn new(error_type: &str, message: &str, status_code: StatusCode) -> Self {
        Self {
            error: error_type.to_string(),
            message: message.to_string(),
            status_code: status_code.as_u16(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = Some(details);
        self
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

/// Error categories for consistent handling
#[derive(Debug, Clone)]
pub enum ErrorCategory {
    /// 400 - Client provided invalid input
    InvalidInput,
    /// 401 - Authentication required
    Unauthorized,
    /// 403 - Access denied
    Forbidden,
    /// 404 - Resource not found
    NotFound,
    /// 409 - Resource conflict (already exists, etc.)
    Conflict,
    /// 422 - Request semantically incorrect
    UnprocessableEntity,
    /// 429 - Rate limit exceeded
    TooManyRequests,
    /// 500 - Internal server error
    InternalError,
    /// 502 - External service unavailable
    ServiceUnavailable,
    /// 503 - Service temporarily unavailable
    TemporarilyUnavailable,
}

impl ErrorCategory {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorCategory::InvalidInput => StatusCode::BAD_REQUEST,
            ErrorCategory::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorCategory::Forbidden => StatusCode::FORBIDDEN,
            ErrorCategory::NotFound => StatusCode::NOT_FOUND,
            ErrorCategory::Conflict => StatusCode::CONFLICT,
            ErrorCategory::UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCategory::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ErrorCategory::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCategory::ServiceUnavailable => StatusCode::BAD_GATEWAY,
            ErrorCategory::TemporarilyUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            ErrorCategory::InvalidInput => "INVALID_INPUT",
            ErrorCategory::Unauthorized => "UNAUTHORIZED",
            ErrorCategory::Forbidden => "FORBIDDEN",
            ErrorCategory::NotFound => "NOT_FOUND",
            ErrorCategory::Conflict => "CONFLICT",
            ErrorCategory::UnprocessableEntity => "UNPROCESSABLE_ENTITY",
            ErrorCategory::TooManyRequests => "TOO_MANY_REQUESTS",
            ErrorCategory::InternalError => "INTERNAL_ERROR",
            ErrorCategory::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            ErrorCategory::TemporarilyUnavailable => "TEMPORARILY_UNAVAILABLE",
        }
    }
}

/// Trait for mapping domain errors to API responses
pub trait IntoApiError {
    fn into_api_error(self) -> ApiErrorResponse;
}

/// Helper functions for creating common error responses
pub mod errors {
    use super::*;

    pub fn invalid_input(message: &str) -> ApiErrorResponse {
        ApiErrorResponse::new(
            ErrorCategory::InvalidInput.error_type(),
            message,
            ErrorCategory::InvalidInput.status_code(),
        )
    }

    pub fn unauthorized(message: &str) -> ApiErrorResponse {
        ApiErrorResponse::new(
            ErrorCategory::Unauthorized.error_type(),
            message,
            ErrorCategory::Unauthorized.status_code(),
        )
    }

    pub fn forbidden(message: &str) -> ApiErrorResponse {
        ApiErrorResponse::new(
            ErrorCategory::Forbidden.error_type(),
            message,
            ErrorCategory::Forbidden.status_code(),
        )
    }

    pub fn not_found(resource: &str) -> ApiErrorResponse {
        ApiErrorResponse::new(
            ErrorCategory::NotFound.error_type(),
            &format!("{} not found", resource),
            ErrorCategory::NotFound.status_code(),
        )
    }

    pub fn conflict(message: &str) -> ApiErrorResponse {
        ApiErrorResponse::new(
            ErrorCategory::Conflict.error_type(),
            message,
            ErrorCategory::Conflict.status_code(),
        )
    }

    pub fn unprocessable_entity(message: &str) -> ApiErrorResponse {
        ApiErrorResponse::new(
            ErrorCategory::UnprocessableEntity.error_type(),
            message,
            ErrorCategory::UnprocessableEntity.status_code(),
        )
    }

    pub fn internal_error() -> ApiErrorResponse {
        ApiErrorResponse::new(
            ErrorCategory::InternalError.error_type(),
            "An internal error occurred. Please try again later.",
            ErrorCategory::InternalError.status_code(),
        )
    }

    pub fn service_unavailable(service: &str) -> ApiErrorResponse {
        ApiErrorResponse::new(
            ErrorCategory::ServiceUnavailable.error_type(),
            &format!("{} service is currently unavailable", service),
            ErrorCategory::ServiceUnavailable.status_code(),
        )
    }

    pub fn temporarily_unavailable(message: &str) -> ApiErrorResponse {
        ApiErrorResponse::new(
            ErrorCategory::TemporarilyUnavailable.error_type(),
            message,
            ErrorCategory::TemporarilyUnavailable.status_code(),
        )
    }
}

/// Macro for easy error logging and conversion
#[macro_export]
macro_rules! log_and_return_error {
    ($error:expr, $api_error:expr) => {{
        tracing::error!("Error occurred: {:?}", $error);
        sentry::capture_error(&$error);
        $api_error
    }};
}