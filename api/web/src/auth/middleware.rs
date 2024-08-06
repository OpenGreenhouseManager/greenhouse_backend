use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tower_cookies::{Cookie, Cookies};

use super::{service, AUTH_TOKEN};
use crate::{auth::Error, AppState};

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
    {
        if service::check_token(&config.service_addresses.auth_service, &token)
            .await
            .is_ok()
        {
            return next.run(req).await;
        }
    }

    sentry::capture_error(&Error::CookieNotFound);
    cookies.remove(Cookie::from(AUTH_TOKEN));
    Response::builder().status(403).body(Body::empty()).unwrap()
}
