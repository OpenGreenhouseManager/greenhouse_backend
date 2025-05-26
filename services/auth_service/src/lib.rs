use crate::router::auth_router::{
    check_token, generate_one_time_token, login, register, register_admin,
};

extern crate diesel_migrations;
pub use self::error::{Error, Result};
use axum::extract::FromRef;
use axum::{Router, routing::post};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use greenhouse_core::auth_service_dto::endpoints;
use serde::Deserialize;
use tower_http::trace::TraceLayer;

pub mod database;
mod error;
mod router;
pub mod token;

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "SERVICE_PORT")]
    pub service_port: u32,
    #[serde(rename = "DATABASE_URL")]
    pub database_url: String,
    #[serde(rename = "JWT_SECRET")]
    pub jwt_secret: String,
    #[serde(rename = "SENTRY_URL")]
    pub sentry_url: String,
}

pub type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: Pool,
}

pub fn app(config: Config, pool: Pool) -> Router {
    let state = AppState { config, pool };

    Router::new()
        .route(endpoints::REGISTER, post(register))
        .route(endpoints::LOGIN, post(login))
        .route(endpoints::CHECK_TOKEN, post(check_token))
        .route(endpoints::ADMIN_REGISTER, post(register_admin))
        .route(
            endpoints::GENERATE_ONE_TIME_TOKEN,
            post(generate_one_time_token),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
