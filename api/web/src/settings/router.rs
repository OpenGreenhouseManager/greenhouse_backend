use crate::helper;
use crate::helper::error::{Error, HttpResult};
use crate::{AppState, auth::AUTH_TOKEN, settings::service};
use axum::routing::post;
use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Response},
};
use greenhouse_core::auth_service_dto::generate_one_time_token::GenerateOneTimeTokenRequestDto;
use tower_cookies::Cookies;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/generate_one_time_token", post(generate_one_time_token))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn generate_one_time_token(
    State(AppState { config }): State<AppState>,
    cookies: Cookies,
    Json(register_request): Json<GenerateOneTimeTokenRequestDto>,
) -> HttpResult<Response> {
    if let Ok(token) = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(Error::CookieNotFound)
    {
        let claims = helper::token::get_claims(token)?;
        if claims.role != "admin" {
            return Err(Error::AdminRoute.into());
        }
    }
    let token = service::generate_one_time_token(
        &config.service_addresses.auth_service,
        &register_request.username,
    )
    .await?;
    Ok(Json(token).into_response())
}
