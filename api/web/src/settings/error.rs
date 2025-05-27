use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::From;
use serde::Serialize;

use crate::helper;

pub(crate) type Result<T> = core::result::Result<T, Error>;

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

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
// endregion: --- Error Boilerplate
