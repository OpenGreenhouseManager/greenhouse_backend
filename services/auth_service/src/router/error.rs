use axum::http::StatusCode;
use derive_more::From;
use serde::Serialize;

use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse, common};
use crate::{database, token};

pub(crate) type Result<T> = core::result::Result<T, HttpErrorResponse<Error>>;

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

impl HttpErrorMapping for Error {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Error::DatabaseConnection => StatusCode::SERVICE_UNAVAILABLE,
            Error::UsernameTaken => StatusCode::CONFLICT,
            Error::UserNotFound => StatusCode::UNAUTHORIZED, // 401 for login failures
            Error::OneTimeToken => StatusCode::BAD_REQUEST,
            Error::User(db_error) => {
                // Map database errors using common patterns
                let error_name = format!("{:?}", db_error);
                common::map_database_error(&error_name)
            }
            Error::Token(token_error) => {
                // Map token errors to appropriate auth status codes
                let error_name = format!("{:?}", token_error);
                common::map_auth_error(&error_name)
            }
        }
    }

    fn to_error_message(&self) -> String {
        match self {
            Error::DatabaseConnection => "Service temporarily unavailable".to_string(),
            Error::UsernameTaken => "Username is already taken".to_string(),
            Error::UserNotFound => "Invalid credentials".to_string(), // Don't reveal if user exists
            Error::OneTimeToken => "Invalid or expired registration token".to_string(),
            Error::User(_) => "Database operation failed".to_string(),
            Error::Token(_) => "Authentication token error".to_string(),
        }
    }

    fn to_error_context(&self) -> Option<serde_json::Value> {
        match self {
            Error::UsernameTaken => Some(serde_json::json!({
                "error_code": "USERNAME_TAKEN",
                "suggestion": "Please choose a different username"
            })),
            Error::UserNotFound => Some(serde_json::json!({
                "error_code": "INVALID_CREDENTIALS"
            })),
            Error::OneTimeToken => Some(serde_json::json!({
                "error_code": "INVALID_TOKEN",
                "suggestion": "Please request a new registration token"
            })),
            _ => None,
        }
    }
}
// endregion: --- Error Boilerplate
