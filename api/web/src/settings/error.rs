use axum::http::StatusCode;
use derive_more::From;
use greenhouse_core::{
    http_error::{HttpErrorMapping, HttpErrorResponse},
    impl_http_error_from,
};
use serde::Serialize;

use crate::helper;

pub(crate) type Result<T> = core::result::Result<T, Error>;
pub(crate) type HttpResult<T> = core::result::Result<T, HttpErrorResponse<Error>>;

#[derive(Debug, Serialize, From)]
pub(crate) enum Error {
    RegisterToken,
    CookieNotFound,
    AdminRoute,
    #[from]
    Token(helper::Error),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl_http_error_from!(Error {
    crate::helper::Error,
});

impl HttpErrorMapping for Error {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Error::RegisterToken => StatusCode::BAD_REQUEST,
            Error::CookieNotFound => StatusCode::UNAUTHORIZED,
            Error::AdminRoute => StatusCode::FORBIDDEN,
            Error::Token(e) => match e {
                helper::Error::InvalidToken => StatusCode::UNAUTHORIZED,
            },
        }
    }

    fn to_error_message(&self) -> String {
        match self {
            Error::RegisterToken => String::from("Register token error"),
            Error::CookieNotFound => String::from("Cookie not found"),
            Error::AdminRoute => String::from("Admin route"),
            Error::Token(e) => match e {
                helper::Error::InvalidToken => String::from("Invalid token"),
            },
        }
    }
}
// endregion: --- Error Boilerplate
