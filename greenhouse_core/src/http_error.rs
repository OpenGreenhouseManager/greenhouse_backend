use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use greenhouse_macro::IntoResponse;
use serde::{Deserialize, Serialize};
use std::fmt;

pub trait HttpErrorMapping: fmt::Display {
    /// Maps this error to an appropriate HTTP status code
    fn to_status_code(&self) -> StatusCode;

    fn to_error_message(&self) -> String {
        self.to_string()
    }

    fn to_error_context(&self) -> Option<serde_json::Value> {
        None
    }
}

#[derive(Debug)]
pub struct HttpErrorResponse<E> {
    pub error: E,
}

impl<E> HttpErrorResponse<E> {
    /// Create a new HttpErrorResponse wrapper
    pub fn new(error: E) -> Self {
        Self { error }
    }
}

impl<E> From<E> for HttpErrorResponse<E> {
    fn from(error: E) -> Self {
        Self::new(error)
    }
}

#[macro_export]
macro_rules! impl_http_error_from {
    ($error_type:ty { $($source_type:ty),+ $(,)? }) => {
        $(
            impl From<$source_type> for $crate::http_error::HttpErrorResponse<$error_type> {
                fn from(error: $source_type) -> Self {
                    $crate::http_error::HttpErrorResponse::new(<$error_type>::from(error))
                }
            }
        )+
    };
}

/// The JSON structure for error responses
#[derive(Serialize, Deserialize, IntoResponse)]
pub struct ErrorResponseBody {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<serde_json::Value>,
}

impl<E> IntoResponse for HttpErrorResponse<E>
where
    E: HttpErrorMapping + fmt::Display + fmt::Debug,
{
    fn into_response(self) -> Response {
        let status_code = self.error.to_status_code();
        let error_message = self.error.to_error_message();
        let context = self.error.to_error_context();

        let body = ErrorResponseBody {
            error: error_message,
            context,
        };

        // Log the error for debugging
        #[cfg(feature = "error_handling")]
        tracing::error!(
            error = ?self.error,
            status_code = ?status_code,
            "HTTP error response"
        );

        (status_code, body).into_response()
    }
}
