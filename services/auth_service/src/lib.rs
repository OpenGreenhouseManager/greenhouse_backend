use crate::router::auth_router::{
    check_token, generate_one_time_token, login, register, register_admin, register_guest,
};

extern crate diesel_migrations;
use axum::extract::FromRef;
use axum::routing::get;
use axum::{Router, routing::post};
use diesel::{Connection, PgConnection};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use greenhouse_core::auth_service_dto::endpoints;
use serde::Deserialize;
use tower_http::trace::TraceLayer;

pub(crate) mod database;
mod router;
pub(crate) mod token;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

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
    #[serde(rename = "ENVIRONMENT", default = "default_environment")]
    pub environment: String,
}

fn default_environment() -> String {
    "development".to_string()
}

pub type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: Pool,
}

pub fn app(config: Config, pool: Pool) -> Router {
    run_migration(&config.database_url);

    let state = AppState { config, pool };

    Router::new()
        .route(endpoints::REGISTER, post(register))
        .route(endpoints::LOGIN, post(login))
        .route(endpoints::CHECK_TOKEN, post(check_token))
        .route(endpoints::ADMIN_REGISTER, post(register_admin))
        .route(endpoints::GUEST_REGISTER, post(register_guest))
        .route(
            endpoints::GENERATE_ONE_TIME_TOKEN,
            post(generate_one_time_token),
        )
        .route("/health", get(|| async {}))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn run_migration(database_url: &str) {
    let mut conn = PgConnection::establish(database_url).unwrap();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
