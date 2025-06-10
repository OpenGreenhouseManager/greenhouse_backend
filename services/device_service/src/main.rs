extern crate diesel_migrations;
use core::panic;
use device_service::{Config, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing::info!("0");
    let config = load_config();
    tracing::info!("1");

    let _guard = sentry::init((
        config.sentry_url.clone(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    tracing::info!("2");

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
            tracing::info!("3");

            let url = format!("0.0.0.0:{}", config.service_port);

            let pool = Pool::builder()
                .build(AsyncDieselConnectionManager::new(
                    config.database_url.clone(),
                ))
                .await
                .unwrap();
            tracing::info!("4");

            let app = device_service::app(config, pool);
            tracing::info!("5");

            let listener = tokio::net::TcpListener::bind(url).await.unwrap();
            tracing::info!("6");
            tracing::info!("listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, app).await.unwrap();
        });
}

fn load_config() -> Config {
    const FILE_PATH: &str = if cfg!(debug_assertions) {
        "services/device_service/config/.env"
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
