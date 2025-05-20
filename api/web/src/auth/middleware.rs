use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tower_cookies::{Cookie, Cookies};

use super::{AUTH_TOKEN, service};
use crate::{AppState, auth::Error};

pub(crate) async fn check_token(
    State(AppState { config }): State<AppState>,
    cookies: Cookies,
    req: Request<Body>,
    next: Next,
) -> Response {
    if let Ok(token) = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(Error::CookieNotFound)
        && service::check_token(&config.service_addresses.auth_service, &token)
            .await
            .is_ok()
    {
        return next.run(req).await;
    }

    cookies.remove(Cookie::from(AUTH_TOKEN));
    tracing::trace!("Invalid Cookie");
    Response::builder().status(403).body(Body::empty()).unwrap()
}
