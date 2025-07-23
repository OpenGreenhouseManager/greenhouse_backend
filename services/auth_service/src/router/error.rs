use crate::{database, token};
use axum::{
    http::StatusCode,
};
use derive_more::From;
use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse};
use greenhouse_core::impl_http_error_from;
use serde::Serialize;

pub(crate) type HttpResult<T> = core::result::Result<T, HttpErrorResponse<Error>>;

#[derive(Debug, Serialize, From)]
pub(crate) enum Error {
    DatabaseConnection,
    UsernameTaken,
    UserNotFound,
    OneTimeToken,
    #[from]
    User(database::Error),
    #[from]
    Token(token::Error),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// Use the macro to implement From for HttpErrorResponse
impl_http_error_from!(Error {
    crate::database::Error,
    crate::token::Error,
});

impl HttpErrorMapping for Error {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Error::DatabaseConnection => StatusCode::INTERNAL_SERVER_ERROR,
            Error::UsernameTaken => StatusCode::BAD_REQUEST,
            Error::UserNotFound => StatusCode::NOT_FOUND,
            Error::OneTimeToken => StatusCode::BAD_REQUEST,
            Error::User(e) => match e {
                database::Error::InvalidHash => StatusCode::BAD_REQUEST,
                database::Error::Token(e) => match e {
                    token::Error::InvalidTime => StatusCode::BAD_REQUEST,
                    token::Error::JwtDecode => StatusCode::BAD_REQUEST,
                    token::Error::JwtEncode => StatusCode::BAD_REQUEST,
                    token::Error::RegisterToken => StatusCode::BAD_REQUEST,
                },
            },
            Error::Token(e) => match e {
                token::Error::InvalidTime => StatusCode::BAD_REQUEST,
                token::Error::JwtDecode => StatusCode::BAD_REQUEST,
                token::Error::JwtEncode => StatusCode::BAD_REQUEST,
                token::Error::RegisterToken => StatusCode::BAD_REQUEST,
            },
        }
    }
}
// endregion: --- Error Boilerplate
