use crate::database;
use axum::http::StatusCode;
use derive_more::From;
use greenhouse_core::{
    http_error::{HttpErrorMapping, HttpErrorResponse},
    impl_http_error_from,
};
use serde::Serialize;

pub(crate) type HttpResult<T> = core::result::Result<T, HttpErrorResponse<Error>>;
pub(crate) type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, From)]
pub(crate) enum Error {
    SmartDeviceNotReachable,
    SmartDeviceResponse,
    #[from]
    Database(database::Error),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl_http_error_from!(Error {
    crate::database::Error,
});

impl HttpErrorMapping for Error {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Error::SmartDeviceNotReachable => StatusCode::SERVICE_UNAVAILABLE,
            Error::SmartDeviceResponse => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Database(e) => match e {
                database::Error::Creation => StatusCode::INTERNAL_SERVER_ERROR,
                database::Error::DatabaseConnection => StatusCode::INTERNAL_SERVER_ERROR,
                database::Error::Find => StatusCode::NOT_FOUND,
            },
        }
    }

    fn to_error_message(&self) -> String {
        match self {
            Error::SmartDeviceNotReachable => String::from("Smart device not reachable"),
            Error::SmartDeviceResponse => String::from("Smart device response error"),
            Error::Database(e) => match e {
                database::Error::Creation => String::from("Database creation error"),
                database::Error::DatabaseConnection => String::from("Database connection error"),
                database::Error::Find => String::from("Database find error"),
            },
        }
    }
}

// endregion: --- Error Boilerplate
