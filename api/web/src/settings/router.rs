use crate::helper;
use crate::settings::{Error, Result};
use crate::{auth::AUTH_TOKEN, settings::service, AppState};
use axum::routing::post;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json, Router,
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
) -> Result<Response> {
    if let Ok(token) = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(Error::CookieNotFound)
    {
        let claims = helper::token::get_claims(token)?;
        if claims.role != "admin" {
            return Err(Error::AdminRoute);
        }
    }
    let token = service::generate_one_time_token(
        &config.service_addresses.auth_service,
        &register_request.username,
    )
    .await?;
    Ok(Json(token).into_response())
}
