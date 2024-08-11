use crate::{auth::Result, AppState};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use greenhouse_core::auth_service_dto::{login::LoginRequestDto, register::RegisterRequestDto};
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

use super::{service, AUTH_TOKEN};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/register", post(api_register_handler))
        .route("/api/login", post(api_login_handler))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn api_register_handler(
    State(AppState { config }): State<AppState>,
    cookie: Cookies,
    Json(register_request): Json<RegisterRequestDto>,
) -> Result<Response> {
    let token = service::register(&config.service_addresses.auth_service, register_request).await?;
    let mut c = Cookie::new(AUTH_TOKEN, token.token.clone());
    c.set_same_site(SameSite::None);
    c.set_http_only(false);
    c.set_path("/");
    cookie.add(c);

    Ok(Json(token).into_response())
}

#[axum::debug_handler]
pub(crate) async fn api_login_handler(
    State(AppState { config }): State<AppState>,
    cookie: Cookies,
    Json(login_request): Json<LoginRequestDto>,
) -> Result<Response> {
    match service::login(&config.service_addresses.auth_service, login_request).await{
        Ok(token) => {
            let mut c = Cookie::new(AUTH_TOKEN, token.token.clone());
            c.set_same_site(SameSite::None);
            c.set_http_only(false);
            c.set_path("/");
            cookie.add(c);
            Ok(Json(token).into_response())
        }
        Err(e)=>{
            cookie.remove(Cookie::from(AUTH_TOKEN));
            Err(e)
        }
    }
}
