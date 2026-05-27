//! Standardised HTTP error response system for Axum handlers.
//!
//! Enabled by the `error_handling` feature of `greenhouse_core`.
//!
//! # Overview
//!
//! Define a custom error enum, implement [`HttpErrorMapping`] on it, and wrap it
//! in [`HttpErrorResponse`] to get a consistent JSON error body with the correct
//! HTTP status code.
//!
//! # Example
//!
//! ```
//! # use axum::http::StatusCode;
//! # use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse};
//! # use std::fmt;
//!
//! #[derive(Debug)]
//! enum MyError {
//!     NotFound,
//!     BadRequest(String),
//! }
//!
//! impl fmt::Display for MyError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         match self {
//!             MyError::NotFound => write!(f, "resource not found"),
//!             MyError::BadRequest(msg) => write!(f, "bad request: {msg}"),
//!         }
//!     }
//! }
//!
//! impl HttpErrorMapping for MyError {
//!     fn to_status_code(&self) -> StatusCode {
//!         match self {
//!             MyError::NotFound => StatusCode::NOT_FOUND,
//!             MyError::BadRequest(_) => StatusCode::BAD_REQUEST,
//!         }
//!     }
//! }
//!
//! let err: HttpErrorResponse<MyError> = MyError::NotFound.into();
//! ```

use axum::{http::StatusCode, response::Response};
use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Maps a domain error to an HTTP status code and optional JSON context.
///
/// Implement this trait on your service's error enum to integrate with
/// [`HttpErrorResponse`]. The `to_error_message` and `to_error_context` methods
/// have default implementations that call `Display` and return `None`
/// respectively.
pub trait HttpErrorMapping: fmt::Display {
    /// Returns the HTTP status code that best represents this error.
    fn to_status_code(&self) -> StatusCode;

    /// Returns the error message string included in the JSON response body.
    ///
    /// Defaults to `self.to_string()`.
    fn to_error_message(&self) -> String {
        self.to_string()
    }

    /// Returns optional structured context included in the JSON response body.
    ///
    /// Defaults to `None` (no context field in the response).
    fn to_error_context(&self) -> Option<serde_json::Value> {
        None
    }
}

/// Wraps a domain error and implements `axum::response::IntoResponse`.
///
/// When returned from an Axum handler, this type serialises the error to a
/// JSON body of the form `{"error": "...", "context": {...}}` with the HTTP
/// status code determined by [`HttpErrorMapping::to_status_code`].
///
/// The `context` field is omitted when [`HttpErrorMapping::to_error_context`]
/// returns `None`.
///
/// # Example
///
/// ```
/// # use axum::http::StatusCode;
/// # use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse};
/// # use std::fmt;
/// # #[derive(Debug)]
/// # enum MyError { NotFound }
/// # impl fmt::Display for MyError {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "not found") }
/// # }
/// # impl HttpErrorMapping for MyError {
/// #     fn to_status_code(&self) -> StatusCode { StatusCode::NOT_FOUND }
/// # }
/// let response = HttpErrorResponse::new(MyError::NotFound);
/// ```
#[derive(Debug)]
pub struct HttpErrorResponse<E> {
    /// The wrapped domain error.
    pub error: E,
}

impl<E> HttpErrorResponse<E> {
    /// Wraps a domain error in an `HttpErrorResponse`.
    pub fn new(error: E) -> Self {
        Self { error }
    }
}

impl<E> From<E> for HttpErrorResponse<E> {
    fn from(error: E) -> Self {
        Self::new(error)
    }
}

/// Implements `From<SourceError>` for `HttpErrorResponse<TargetError>` by
/// first converting `SourceError` into `TargetError`.
///
/// Use this macro to avoid boilerplate when multiple source error types
/// (e.g. `diesel::result::Error`, `uuid::Error`) should all map to a single
/// service error type.
///
/// # Example
///
/// ```rust,ignore
/// # use axum::http::StatusCode;
/// # use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse};
/// # use greenhouse_core::impl_http_error_from;
/// # use std::fmt;
/// # #[derive(Debug)]
/// # enum MyError { ParseError(std::num::ParseIntError) }
/// # impl fmt::Display for MyError {
/// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "error") }
/// # }
/// # impl HttpErrorMapping for MyError {
/// #     fn to_status_code(&self) -> StatusCode { StatusCode::BAD_REQUEST }
/// # }
/// # impl From<std::num::ParseIntError> for MyError {
/// #     fn from(e: std::num::ParseIntError) -> Self { MyError::ParseError(e) }
/// # }
/// impl_http_error_from!(MyError { std::num::ParseIntError });
/// // Now `?` on a `ParseIntError` in a handler returning `HttpErrorResponse<MyError>` works.
/// ```
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

/// JSON response body for HTTP errors.
#[derive(Serialize, Deserialize, IntoJsonResponse)]
pub struct ErrorResponseBody {
    /// Human-readable error message.
    pub error: String,
    /// Optional structured context (omitted when `None`).
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<serde_json::Value>,
}

impl<E> axum::response::IntoResponse for HttpErrorResponse<E>
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
