use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::From;
use serde::Serialize;
use greenhouse_core::error::{ApiErrorResponse, IntoApiError, errors};

pub(crate) type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, From)]
pub(crate) enum Error {
    InternalError,
    Unauthorized,
    InvalidToken,
    #[from]
    RequestError(reqwest::Error),
}

impl IntoApiError for Error {
    fn into_api_error(self) -> ApiErrorResponse {
        match self {
            Error::InternalError => {
                tracing::error!("Internal error in helper API: {:?}", self);
                sentry::capture_error(&self as &dyn std::error::Error);
                errors::internal_error()
            }
            Error::Unauthorized => {
                errors::unauthorized("Authentication required")
            }
            Error::InvalidToken => {
                errors::unauthorized("Invalid or expired token")
            }
            Error::RequestError(req_err) => {
                tracing::error!("Request error: {:?}", req_err);
                sentry::capture_error(&req_err);
                
                if req_err.is_timeout() {
                    errors::temporarily_unavailable("Service is temporarily unavailable")
                } else if req_err.is_connect() {
                    errors::service_unavailable("Service")
                } else if req_err.is_status() {
                    if let Some(status) = req_err.status() {
                        match status {
                            reqwest::StatusCode::UNAUTHORIZED => {
                                errors::unauthorized("Authentication required")
                            }
                            reqwest::StatusCode::FORBIDDEN => {
                                errors::forbidden("Access denied")
                            }
                            _ => errors::internal_error()
                        }
                    } else {
                        errors::internal_error()
                    }
                } else {
                    errors::internal_error()
                }
            }
        }
    }
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.into_api_error().into_response()
    }
}
// endregion: --- Error Boilerplate
