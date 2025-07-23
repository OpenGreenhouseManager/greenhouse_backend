//! HTTP Error Mapping System
//!
//! This module provides a centralized way to map errors to HTTP status codes and
//! create consistent HTTP responses across the greenhouse backend services.
//!
//! # Usage
//!
//! Implement the `HttpErrorMapping` trait on your error enums:
//!
//! ```rust
//! use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse, HttpResult};
//! use axum::http::StatusCode;
//!
//! #[derive(Debug)]
//! enum MyError {
//!     NotFound,
//!     InvalidInput,
//!     DatabaseError,
//! }
//!
//! impl HttpErrorMapping for MyError {
//!     fn to_status_code(&self) -> StatusCode {
//!         match self {
//!             MyError::NotFound => StatusCode::NOT_FOUND,
//!             MyError::InvalidInput => StatusCode::BAD_REQUEST,
//!             MyError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
//!         }
//!     }
//! }
//!
//! // Use HttpResult for cleaner handler signatures
//! async fn my_handler() -> HttpResult<Json<Data>, MyError> {
//!     let data = get_data().map_err(HttpErrorResponse::new)?;
//!     Ok(Json(data))
//! }
//!
//! // Or use it with any success type
//! fn process_data() -> HttpResult<i32, MyError> {
//!     // ... processing logic
//!     Ok(42)
//! }
//! ```

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::fmt;

/// Trait for mapping errors to HTTP status codes
///
/// Implement this trait on your error enums to define how they should be
/// converted to HTTP status codes.
pub trait HttpErrorMapping: fmt::Display {
    /// Maps this error to an appropriate HTTP status code
    fn to_status_code(&self) -> StatusCode;

    /// Optional: Provide a custom error message for the HTTP response
    ///
    /// If not overridden, uses the error's Display implementation
    fn to_error_message(&self) -> String {
        self.to_string()
    }

    /// Optional: Provide additional context or metadata for the error response
    ///
    /// This can be used to include error codes, additional details, etc.
    fn to_error_context(&self) -> Option<serde_json::Value> {
        None
    }
}

/// A wrapper type that implements IntoResponse for errors that implement HttpErrorMapping
///
/// This provides a consistent way to convert errors into HTTP responses across
/// all services in the greenhouse backend.
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

/// Macro to implement From for HttpErrorResponse for common error types
/// 
/// This macro helps reduce boilerplate when implementing From for
/// HttpErrorResponse when you have From implementations to your main error type.
/// 
/// # Example
/// 
/// ```rust
/// use greenhouse_core::impl_http_error_from;
/// 
/// #[derive(Debug)]
/// enum MyError {
///     DatabaseError(database::Error),
///     TokenError(token::Error),
/// }
/// 
/// impl_http_error_from!(MyError {
///     database::Error,
///     token::Error,
/// });
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

/// The JSON structure for error responses
#[derive(Serialize)]
struct ErrorResponseBody {
    error: String,
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

        (status_code, Json(body)).into_response()
    }
}

/// Macro to implement HttpErrorMapping with common patterns
///
/// This macro helps reduce boilerplate when implementing HttpErrorMapping
/// for error enums that follow common patterns.
///
/// # Example
///
/// ```rust
/// use greenhouse_core::impl_http_error_mapping;
///
/// #[derive(Debug)]
/// enum DatabaseError {
///     NotFound,
///     Creation,
///     DatabaseConnection,
/// }
///
/// impl_http_error_mapping!(DatabaseError {
///     NotFound => NOT_FOUND,
///     Creation => INTERNAL_SERVER_ERROR,
///     DatabaseConnection => SERVICE_UNAVAILABLE,
/// });
/// ```
#[macro_export]
macro_rules! impl_http_error_mapping {
    ($error_type:ty { $($variant:ident => $status:ident),+ $(,)? }) => {
        impl $crate::http_error::HttpErrorMapping for $error_type {
            fn to_status_code(&self) -> axum::http::StatusCode {
                match self {
                    $(
                        Self::$variant => axum::http::StatusCode::$status,
                    )+
                }
            }
        }
    };
}

/// Macro to implement HttpErrorMapping using common error mappers
///
/// This macro allows you to use the predefined common error mappers
/// for standard error patterns.
///
/// # Example
///
/// ```rust
/// use greenhouse_core::impl_http_error_mapping_with_common;
///
/// #[derive(Debug)]
/// enum MyError {
///     DatabaseError(String),
///     AuthError(String),
///     CustomError,
/// }
///
/// impl_http_error_mapping_with_common!(MyError {
///     DatabaseError(ref e) => database(e),
///     AuthError(ref e) => auth(e),
///     CustomError => BAD_REQUEST,
/// });
/// ```
#[macro_export]
macro_rules! impl_http_error_mapping_with_common {
    ($error_type:ty { $($variant:pat => $mapper:ident($($args:expr),*)),+ $(,)? }) => {
        impl $crate::http_error::HttpErrorMapping for $error_type {
            fn to_status_code(&self) -> axum::http::StatusCode {
                match self {
                    $(
                        $variant => $crate::http_error::common::$mapper($($args),*),
                    )+
                }
            }
        }
    };
    ($error_type:ty { $($variant:pat => $status:ident),+ $(,)? }) => {
        impl $crate::http_error::HttpErrorMapping for $error_type {
            fn to_status_code(&self) -> axum::http::StatusCode {
                match self {
                    $(
                        $variant => axum::http::StatusCode::$status,
                    )+
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum TestError {
        NotFound,
        InvalidInput,
        DatabaseError,
    }

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                TestError::NotFound => write!(f, "Resource not found"),
                TestError::InvalidInput => write!(f, "Invalid input provided"),
                TestError::DatabaseError => write!(f, "Database operation failed"),
            }
        }
    }

    impl HttpErrorMapping for TestError {
        fn to_status_code(&self) -> StatusCode {
            match self {
                TestError::NotFound => StatusCode::NOT_FOUND,
                TestError::InvalidInput => StatusCode::BAD_REQUEST,
                TestError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
    }

    #[test]
    fn test_error_status_mapping() {
        assert_eq!(TestError::NotFound.to_status_code(), StatusCode::NOT_FOUND);
        assert_eq!(
            TestError::InvalidInput.to_status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            TestError::DatabaseError.to_status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
