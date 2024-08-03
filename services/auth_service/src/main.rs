pub use self::error::{Error, Result};

extern crate diesel_migrations;
use crate::diesel_migrations::MigrationHarness;
use axum::{extract::FromRef, routing::post, Router};
use core::panic;
use diesel::{Connection, PgConnection};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use router::auth_router::{check_token, login, register};
use serde::Deserialize;

pub mod database;
mod error;
mod router;
pub mod user_token;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Clone, Deserialize)]
struct Config {
    #[serde(rename = "SERVICE_PORT")]
    service_port: u32,
    #[serde(rename = "DATABASE_URL")]
    database_url: String,
    #[serde(rename = "JWT_SECRET")]
    jwt_secret: String,
}

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(FromRef, Clone)]
struct AppState {
    config: Config,
    pool: Pool,
}

#[tokio::main]
async fn main() {
    let file_path = if cfg!(debug_assertions) {
        "services/auth_service/config/.env"
    } else {
        "config/.env"
    };

    let config: Config = match std::fs::File::open(file_path) {
        Ok(f) => match serde_yaml::from_reader(f) {
            Ok(config) => config,
            Err(e) => {
                panic!("Failed to read config file: {}", e)
            }
        },
        Err(e) => {
            panic!("Failed to open config file at: {}", e)
        }
    };

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
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/check_token", post(check_token))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

fn run_migration(database_url: &str) {
    let mut conn = PgConnection::establish(database_url).unwrap();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
