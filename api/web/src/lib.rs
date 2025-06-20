use auth::middleware::check_token;
use axum::extract::FromRef;
use axum::{Router, middleware};
use reqwest::{
    Method,
    header::{ACCEPT, AUTHORIZATION},
};
use serde::Deserialize;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub(crate) mod alert;
pub(crate) mod auth;
pub(crate) mod diary;
pub(crate) mod device;
pub(crate) mod helper;
pub(crate) mod settings;
pub(crate) mod test;

#[derive(Clone, Deserialize)]
pub struct ServiceAddresses {
    #[serde(rename = "AUTH_SERVICE")]
    pub auth_service: String,
    #[serde(rename = "DATA_STORAGE_SERVICE")]
    pub data_storage_service: String,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "API_PORT")]
    pub api_port: u32,
    #[serde(rename = "SERVICE_ADDRESSES")]
    pub service_addresses: ServiceAddresses,
    #[serde(rename = "SENTRY_URL")]
    pub sentry_url: String,
}

#[derive(FromRef, Clone)]
struct AppState {
    config: Config,
}

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
        .nest("/api/device", device::router::routes(state.clone()))
        .layer(middleware::from_fn_with_state(state.clone(), check_token))
        .merge(auth::router::routes(state))
        .layer(CookieManagerLayer::new())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
