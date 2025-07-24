use axum::http::StatusCode;
use derive_more::From;
use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse};
use serde::Serialize;

pub(crate) type Result<T> = core::result::Result<T, Error>;
pub(crate) type HttpResult<T> = core::result::Result<T, HttpErrorResponse<Error>>;

#[derive(Debug, Serialize, From)]
pub(crate) enum Error {
    Internal,
    Unauthorized,
    CookieNotFound,
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
            Error::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::CookieNotFound => StatusCode::UNAUTHORIZED,
        }
    }

    fn to_error_message(&self) -> String {
        match self {
            Error::Internal => String::from("Internal error"),
            Error::Unauthorized => String::from("Unauthorized"),
            Error::CookieNotFound => String::from("Cookie not found"),
        }
    }
}
// endregion: --- Error Boilerplate
