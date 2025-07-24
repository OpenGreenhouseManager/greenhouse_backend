use axum::http::StatusCode;
use derive_more::From;
use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse};
use serde::Serialize;

pub(crate) type Result<T> = core::result::Result<T, Error>;
pub(crate) type HttpResult<T> = core::result::Result<T, HttpErrorResponse<Error>>;

#[derive(Debug, Serialize, From)]
pub(crate) enum Error {
    InternalError,
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
            Error::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn to_error_message(&self) -> String {
        match self {
            Error::InternalError => String::from("Internal error"),
        }
    }
}

// endregion: --- Error Boilerplate
