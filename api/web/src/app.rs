use auth::middleware::check_token;
use axum::{Router, middleware};
use reqwest::{
    Method,
    header::{ACCEPT, AUTHORIZATION},
};
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{AppState, Config, alert, auth, diary, settings, test};

pub fn app(config: Config) -> Router {
    let state = AppState { config };

    let cors = CorsLayer::new()
        .allow_headers([AUTHORIZATION, ACCEPT, reqwest::header::CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin([
            "0.0.0.0".parse().unwrap(),
            "http://localhost:4200".parse().unwrap(),
            "https://localhost:5001".parse().unwrap(),
        ]);
    Router::new()
        .nest("/api", test::router::routes(state.clone()))
        .nest("/api/settings", settings::router::routes(state.clone()))
        .nest("/api/diary", diary::router::routes(state.clone()))
        .nest("/api/alert", alert::router::routes(state.clone()))
        .layer(middleware::from_fn_with_state(state.clone(), check_token))
        .merge(auth::router::routes(state))
        .layer(CookieManagerLayer::new())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
