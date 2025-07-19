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
    NotFound,
    ServiceUnavailable,
    InvalidRequest,
    #[from]
    RequestError(reqwest::Error),
}

impl IntoApiError for Error {
    fn into_api_error(self) -> ApiErrorResponse {
        match self {
            Error::InternalError => {
                tracing::error!("Internal error in alert API: {:?}", self);
                sentry::capture_error(&self as &dyn std::error::Error);
                errors::internal_error()
            }
            Error::NotFound => {
                errors::not_found("Alert")
            }
            Error::ServiceUnavailable => {
                errors::service_unavailable("Data storage")
            }
            Error::InvalidRequest => {
                errors::invalid_input("Invalid request data")
            }
            Error::RequestError(req_err) => {
                tracing::error!("Request error: {:?}", req_err);
                sentry::capture_error(&req_err);
                
                if req_err.is_timeout() {
                    errors::temporarily_unavailable("Data storage service is temporarily unavailable")
                } else if req_err.is_connect() {
                    errors::service_unavailable("Data storage")
                } else if req_err.is_status() {
                    if let Some(status) = req_err.status() {
                        match status {
                            reqwest::StatusCode::NOT_FOUND => errors::not_found("Alert"),
                            reqwest::StatusCode::BAD_REQUEST => {
                                errors::invalid_input("Invalid request data")
                            }
                            reqwest::StatusCode::UNAUTHORIZED => {
                                errors::unauthorized("Authentication required")
                            }
                            reqwest::StatusCode::FORBIDDEN => {
                                errors::forbidden("Access denied")
                            }
                            _ => errors::service_unavailable("Data storage")
                        }
                    } else {
                        errors::service_unavailable("Data storage")
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
