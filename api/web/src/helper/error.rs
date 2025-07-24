use axum::http::StatusCode;
use greenhouse_core::http_error::{HttpErrorMapping, HttpErrorResponse};

pub(crate) type Result<T> = core::result::Result<T, Error>;
pub(crate) type HttpResult<T> = core::result::Result<T, HttpErrorResponse<Error>>;

#[derive(Debug)]
pub(crate) struct ApiError {
    pub(crate) status: StatusCode,
    pub(crate) message: String,
}

#[derive(Debug)]
pub(crate) enum Error {
    CookieNotFound,
    InvalidToken,
    AdminRoute,
    Api(ApiError),
    Request(reqwest::Error),
    Json(reqwest::Error),
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
            Error::CookieNotFound => StatusCode::UNAUTHORIZED,
            Error::InvalidToken => StatusCode::UNAUTHORIZED,
            Error::AdminRoute => StatusCode::UNAUTHORIZED,
            Error::Api(e) => e.status,
            Error::Request(e) => match e.status() {
                Some(status) => status,
                None => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Error::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn to_error_message(&self) -> String {
        match self {
            Error::CookieNotFound => String::from("Cookie not found"),
            Error::InvalidToken => String::from("Invalid token"),
            Error::AdminRoute => String::from("Admin route"),
            Error::Api(e) => e.message.clone(),
            Error::Request(e) => e.to_string(),
            Error::Json(e) => e.to_string(),
        }
    }
}
// endregion: --- Error Boilerplate
