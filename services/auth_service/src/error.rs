use std::sync::Arc;

use crate::{database, user_token};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::From;
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, From)]
pub enum Error {
    DatabaseConnection,
    #[from]
    User(database::Error),
    #[from]
    JWT(user_token::Error),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // Create a placeholder Axum reponse.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the reponse.
        response.extensions_mut().insert(Arc::new(self));

        response
    }
}
