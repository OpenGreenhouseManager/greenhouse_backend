use crate::{auth::Result, AppState};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use greenhouse_core::auth_service_dto::{login::LoginRequestDto, register::RegisterRequestDto};

use super::service;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/test", get(api_test_handler))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn api_test_handler(
    State(AppState { config }): State<AppState>,
) -> Result<Response> {
    "worked".into_response()
}
