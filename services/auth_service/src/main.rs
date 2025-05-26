extern crate diesel_migrations;
use crate::diesel_migrations::MigrationHarness;
use auth_service::{Config, Pool};
use core::panic;
use diesel::{Connection, PgConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

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
                        "auth_service=debug,tower_http=debug,axum::rejection=trace".into()
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

            let app = auth_service::app(config, pool);

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
        "services/auth_service/config/.env"
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
