extern crate diesel_migrations;
use std::future::ready;
use std::time::Duration;

use axum::extract::FromRef;
use axum::{Router, routing::get};
use diesel::{Connection, PgConnection};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use serde::Deserialize;
use tower_http::trace::TraceLayer;

pub(crate) mod database;
mod router;
mod scrape_service;

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
    let recorder_handle = setup_metrics_recorder();

    scrape_service::start_scrape_devices(state.clone());
    Router::new()
        .route("/metrics", get(move || ready(recorder_handle.render())))
        .merge(router::device_router::routes(state.clone()))
        .route("/health", get(|| async {}))
        .layer(TraceLayer::new_for_http())
}

pub fn test_app(config: Config, pool: Pool) -> Router {
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

fn setup_metrics_recorder() -> PrometheusHandle {
    let recorder_handle = PrometheusBuilder::new().install_recorder().unwrap();

    let upkeep_handle = recorder_handle.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            upkeep_handle.run_upkeep();
        }
    });

    recorder_handle
}
