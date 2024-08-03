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
        .route("/api/register", post(api_register_handler))
        .route("/api/login", post(api_login_handler))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn api_register_handler(
    State(AppState { config }): State<AppState>,
    Json(register_request): Json<RegisterRequestDto>,
) -> Result<Response> {
    Ok(
        Json(service::register(&config.service_addresses.auth_service, register_request).await?)
            .into_response(),
    )
}

#[axum::debug_handler]
pub(crate) async fn api_login_handler(
    State(AppState { config }): State<AppState>,
    Json(login_request): Json<LoginRequestDto>,
) -> Result<Response> {
    Ok(
        Json(service::login(&config.service_addresses.auth_service, login_request).await?)
            .into_response(),
    )
}
