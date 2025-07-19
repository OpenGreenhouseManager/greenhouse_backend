use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::From;
use serde::Serialize;
use greenhouse_core::error::{ApiErrorResponse, IntoApiError, errors};

use crate::{database, token};

pub(crate) type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, From)]
pub(crate) enum Error {
    DatabaseConnection,
    UsernameTaken,
    UserNotFound,
    OneTimeToken,
    InvalidCredentials,
    #[from]
    User(database::Error),
    #[from]
    Token(token::Error),
}

impl IntoApiError for Error {
    fn into_api_error(self) -> ApiErrorResponse {
        match self {
            Error::DatabaseConnection => {
                tracing::error!("Database connection error: {:?}", self);
                sentry::capture_error(&self as &dyn std::error::Error);
                errors::internal_error()
            }
            Error::UsernameTaken => {
                errors::conflict("Username is already taken")
            }
            Error::UserNotFound => {
                errors::unauthorized("Invalid username or password")
            }
            Error::OneTimeToken => {
                errors::unauthorized("Invalid or expired one-time token")
            }
            Error::InvalidCredentials => {
                errors::unauthorized("Invalid username or password")
            }
            Error::User(database::Error::InvalidHash) => {
                tracing::error!("Password hashing error: {:?}", self);
                sentry::capture_error(&self as &dyn std::error::Error);
                errors::internal_error()
            }
            Error::User(database::Error::Token(token_error)) => {
                match token_error {
                    token::Error::JwtEncode | token::Error::JwtDecode => {
                        tracing::error!("JWT processing error: {:?}", self);
                        sentry::capture_error(&self as &dyn std::error::Error);
                        errors::internal_error()
                    }
                    token::Error::InvalidTime => {
                        tracing::error!("Time processing error: {:?}", self);
                        sentry::capture_error(&self as &dyn std::error::Error);
                        errors::internal_error()
                    }
                    token::Error::RegisterToken => {
                        errors::unauthorized("Invalid or expired token")
                    }
                }
            }
            Error::Token(token_error) => {
                match token_error {
                    token::Error::JwtEncode | token::Error::JwtDecode => {
                        tracing::error!("JWT processing error: {:?}", self);
                        sentry::capture_error(&self as &dyn std::error::Error);
                        errors::internal_error()
                    }
                    token::Error::InvalidTime => {
                        tracing::error!("Time processing error: {:?}", self);
                        sentry::capture_error(&self as &dyn std::error::Error);
                        errors::internal_error()
                    }
                    token::Error::RegisterToken => {
                        errors::unauthorized("Invalid or expired token")
                    }
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
