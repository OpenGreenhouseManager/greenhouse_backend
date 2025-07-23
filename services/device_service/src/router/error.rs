use crate::database;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::From;
use serde::Serialize;

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

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::SmartDeviceNotReachable => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            Error::SmartDeviceResponse => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            Error::Database(e) => match e {
                database::Error::Creation => {
                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                }
                database::Error::DatabaseConnection => {
                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                }
                database::Error::Find => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
            },
        }
    }
}
// endregion: --- Error Boilerplate
