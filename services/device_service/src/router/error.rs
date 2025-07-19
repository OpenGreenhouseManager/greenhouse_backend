use crate::database;
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
    SmartDeviceNotReachable,
    SmartDeviceResponse,
    InvalidDeviceData,
    #[from]
    Database(database::Error),
}

impl IntoApiError for Error {
    fn into_api_error(self) -> ApiErrorResponse {
        match self {
            Error::SmartDeviceNotReachable => {
                tracing::warn!("Smart device not reachable: {:?}", self);
                errors::service_unavailable("Device")
            }
            Error::SmartDeviceResponse => {
                tracing::warn!("Invalid smart device response: {:?}", self);
                errors::service_unavailable("Device")
            }
            Error::InvalidDeviceData => {
                errors::invalid_input("Invalid device data provided")
            }
            Error::Database(database::Error::Creation) => {
                tracing::error!("Database creation error: {:?}", self);
                sentry::capture_error(&self as &dyn std::error::Error);
                errors::internal_error()
            }
            Error::Database(database::Error::DatabaseConnection) => {
                tracing::error!("Database connection error: {:?}", self);
                sentry::capture_error(&self as &dyn std::error::Error);
                errors::internal_error()
            }
            Error::Database(database::Error::Find) => {
                errors::not_found("Device")
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
