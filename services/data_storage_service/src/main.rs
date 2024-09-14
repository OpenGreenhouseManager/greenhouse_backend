pub use self::error::{Error, Result};

extern crate diesel_migrations;
use crate::diesel_migrations::MigrationHarness;
use axum::{extract::FromRef, Router};
use core::panic;
use diesel::{Connection, PgConnection};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod database;
mod error;
mod router;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Clone, Deserialize)]
struct Config {
    #[serde(rename = "SERVICE_PORT")]
    service_port: u32,
    #[serde(rename = "DATABASE_URL")]
    database_url: String,
    #[serde(rename = "JWT_SECRET")]
    sentry_url: String,
}

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(FromRef, Clone)]
struct AppState {
    config: Config,
    pool: Pool,
}

fn main() {
    let config = load_config();

    let _guard = sentry::init((
        config.sentry_url.clone(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                        "data_storage_service=debug,tower_http=debug,axum::rejection=trace".into()
                    }),
                )
                .with(tracing_subscriber::fmt::layer())
                .init();

            run_migration(&config.database_url);

            let url = format!("0.0.0.0:{}", config.service_port);

            let pool = Pool::builder()
                .build(AsyncDieselConnectionManager::new(
                    config.database_url.clone(),
                ))
                .await
                .unwrap();

            let state = AppState { config, pool };

            let app = Router::new()
                .layer(TraceLayer::new_for_http())
                .with_state(state);

            let listener = tokio::net::TcpListener::bind(url).await.unwrap();
            tracing::info!("listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, app).await.unwrap();
        });
}

fn run_migration(database_url: &str) {
    let mut conn = PgConnection::establish(database_url).unwrap();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

fn load_config() -> Config {
    const FILE_PATH: &str = if cfg!(debug_assertions) {
        "services/data_storage_service/config/.env"
    } else {
        "config/.env"
    };

    match std::fs::File::open(FILE_PATH) {
        Ok(f) => match serde_yaml::from_reader(f) {
            Ok(config) => config,
            Err(e) => {
                panic!("Failed to read config file: {}", e)
            }
        },
        Err(_) => {
            panic!("Failed to open config file at: {}", FILE_PATH)
        }
    }
}
