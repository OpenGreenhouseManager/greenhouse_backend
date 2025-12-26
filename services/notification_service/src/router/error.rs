use axum::http::StatusCode;
use derive_more::From;
use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse};
use greenhouse_core::impl_http_error_from;
use serde::Serialize;

pub(crate) type HttpResult<T> = core::result::Result<T, HttpErrorResponse<Error>>;

#[derive(Debug, Serialize, From)]
pub(crate) enum Error {
    DatabaseConnection,
    UserNotFound
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl HttpErrorMapping for Error {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Error::DatabaseConnection => StatusCode::INTERNAL_SERVER_ERROR,
            Error::UserNotFound => StatusCode::NOT_FOUND,
        }
    }

    fn to_error_message(&self) -> String {
        match self {
            Error::DatabaseConnection => String::from("Database connection error"),
            Error::UserNotFound => String::from("Username or password incorrect"),
        }
    }
}
// endregion: --- Error Boilerplate
