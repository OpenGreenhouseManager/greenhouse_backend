use axum::{http::StatusCode, response::Response};
use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Controls whether and at what severity an error is sent to Sentry.
/// Returned by `HttpErrorMapping::sentry_level`; kept free of the sentry dep
/// so every error type can override it without a feature gate.
pub enum SentryLevel {
    Error,
    Warning,
}

pub trait HttpErrorMapping: fmt::Display {
    /// Maps this error to an appropriate HTTP status code
    fn to_status_code(&self) -> StatusCode;

    fn to_error_message(&self) -> String {
        self.to_string()
    }

    fn to_error_context(&self) -> Option<serde_json::Value> {
        None
    }

    /// Whether and at what level to report this error to Sentry.
    /// Default: 5xx → Error, 4xx → None (silent).
    /// Override per-variant to suppress noisy-but-expected errors (e.g. failed logins)
    /// or to promote mis-classified errors (e.g. a 400 that is actually a server fault).
    fn sentry_level(&self) -> Option<SentryLevel> {
        if self.to_status_code().is_server_error() {
            Some(SentryLevel::Error)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct HttpErrorResponse<E> {
    pub error: E,
    /// Stacktrace captured at the point of error conversion (the `?` site).
    /// Staged here so `into_response` can decide whether to send based on `sentry_level`.
    #[cfg(feature = "sentry")]
    sentry_stacktrace: Option<sentry::protocol::Stacktrace>,
}

impl<E> HttpErrorResponse<E> {
    pub fn new(error: E) -> Self {
        Self {
            error,
            #[cfg(feature = "sentry")]
            sentry_stacktrace: sentry::integrations::backtrace::current_stacktrace(),
        }
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
#[derive(Serialize, Deserialize, IntoJsonResponse)]
pub struct ErrorResponseBody {
    pub error: String,
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

        #[cfg(feature = "sentry")]
        if let Some(level) = self.error.sentry_level() {
            let sentry_level = match level {
                SentryLevel::Error => sentry::Level::Error,
                SentryLevel::Warning => sentry::Level::Warning,
            };
            let mut event = sentry::protocol::Event::new();
            event.stacktrace = self.sentry_stacktrace;
            event.level = sentry_level;
            event.message = Some(format!("{:?}", self.error));
            sentry::capture_event(event);
        }

        #[cfg(feature = "error_handling")]
        if status_code.is_server_error() {
            tracing::error!(
                error = ?self.error,
                status_code = ?status_code,
                "HTTP error response"
            );
        } else {
            tracing::warn!(
                error = ?self.error,
                status_code = ?status_code,
                "HTTP error response"
            );
        }

        (status_code, ErrorResponseBody { error: error_message, context }).into_response()
    }
}
