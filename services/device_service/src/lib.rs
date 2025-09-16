extern crate diesel_migrations;
use axum::Router;
use axum::extract::FromRef;
use diesel::{Connection, PgConnection};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use serde::Deserialize;
use tower_http::trace::TraceLayer;

pub(crate) mod database;
mod router;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "SERVICE_PORT")]
    pub service_port: u32,
    #[serde(rename = "SCRIPTING_API")]
    pub scripting_api: String,
    #[serde(rename = "SCRIPTING_SERVICE")]
    pub scripting_service: String,
    #[serde(rename = "DATABASE_URL")]
    pub database_url: String,
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
        .merge(router::device_router::routes(state.clone()))
        .layer(TraceLayer::new_for_http())
}

fn run_migration(database_url: &str) {
    let mut conn = PgConnection::establish(database_url).unwrap();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
