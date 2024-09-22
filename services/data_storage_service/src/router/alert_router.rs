use super::error::Result;
use axum::{
    routing::{get, post, put},
    Router,
};
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
pub struct Params {
    start: String,
    end: String,
}

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_diary_entry))
        .route("/:id", put(update_diary_entry))
        .route("/:id", get(get_diary_entry))
        .route("/:start/:end", get(get_diary))
        .with_state(state)
}
