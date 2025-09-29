use crate::{AppState, helper::error::HttpResult};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use greenhouse_core::auth_service_dto::{
    login::LoginRequestDto,
    register::RegisterRequestDto,
    user_preferences::{UserPreferencesRequestDto, UserPreferencesResponseDto},
};
use tower_cookies::{Cookie, Cookies, cookie::SameSite};
use uuid::Uuid;

use super::{AUTH_TOKEN, service};

pub(crate) fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/register", post(api_register_handler))
        .route("/api/login", post(api_login_handler))
        .with_state(state)
}

pub(crate) fn user_routes(state: AppState) -> Router {
    Router::new()
        .route("/{user_id}/preferences", get(api_get_user_preferences_handler))
        .route("/{user_id}/preferences", post(api_set_user_preferences_handler))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn api_register_handler(
    State(AppState { config }): State<AppState>,
    cookie: Cookies,
    Json(register_request): Json<RegisterRequestDto>,
) -> HttpResult<Response> {
    tracing::trace!(
        "Registering user with username: {} and one-time token: {}",
        register_request.username,
        register_request.one_time_token
    );
    match service::register(&config.service_addresses.auth_service, register_request).await {
        Ok(token) => {
            let mut c = Cookie::new(AUTH_TOKEN, token.token.clone());
            c.set_same_site(SameSite::None);
            c.set_http_only(false);
            c.set_secure(true);
            c.set_path("/");
            cookie.add(c);
            Ok(Json(token).into_response())
        }
        Err(e) => {
            cookie.remove(Cookie::from(AUTH_TOKEN));
            tracing::error!("Error during registration: {:?}", e);
            Err(e.into())
        }
    }
}

#[axum::debug_handler]
pub(crate) async fn api_login_handler(
    State(AppState { config }): State<AppState>,
    cookie: Cookies,
    Json(login_request): Json<LoginRequestDto>,
) -> HttpResult<Response> {
    tracing::trace!("Logging in user with username: {}", login_request.username,);
    match service::login(&config.service_addresses.auth_service, login_request).await {
        Ok(token) => {
            let mut c = Cookie::new(AUTH_TOKEN, token.token.clone());
            c.set_same_site(SameSite::None);
            c.set_http_only(false);
            c.set_secure(true);
            c.set_path("/");
            cookie.add(c);
            Ok(Json(token).into_response())
        }
        Err(e) => {
            cookie.remove(Cookie::from(AUTH_TOKEN));
            tracing::error!("Error during login: {:?}", e);
            Err(e.into())
        }
    }
}

#[axum::debug_handler]
pub(crate) async fn api_get_user_preferences_handler(
    State(AppState { config }): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> HttpResult<UserPreferencesResponseDto> {
    let resp =
        service::get_user_preferences(&config.service_addresses.auth_service, user_id).await?;
    Ok(resp)
}

#[axum::debug_handler]
pub(crate) async fn api_set_user_preferences_handler(
    State(AppState { config }): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(user_preferences): Json<UserPreferencesRequestDto>,
) -> HttpResult<UserPreferencesResponseDto> {
    let resp = service::set_user_preferences(
        &config.service_addresses.auth_service,
        user_id,
        user_preferences,
    )
    .await?;
    Ok(resp)
}
