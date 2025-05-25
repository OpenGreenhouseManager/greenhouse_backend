use axum::extract::FromRef;
use serde::Deserialize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
pub mod alert;
mod app;
pub mod auth;
pub mod diary;
pub mod helper;
pub mod settings;
pub mod test;

#[derive(Clone, Deserialize)]
struct ServiceAddresses {
    #[serde(rename = "AUTH_SERVICE")]
    auth_service: String,
    #[serde(rename = "DATA_STORAGE_SERVICE")]
    data_storage_service: String,
}

#[derive(Clone, Deserialize)]
struct Config {
    #[serde(rename = "API_PORT")]
    api_port: u32,
    #[serde(rename = "SERVICE_ADDRESSES")]
    service_addresses: ServiceAddresses,
    #[serde(rename = "SENTRY_URL")]
    sentry_url: String,
}

#[derive(FromRef, Clone)]
struct AppState {
    config: Config,
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
                        "web_api=debug,tower_http=debug,axum::rejection=trace".into()
                    }),
                )
                .with(tracing_subscriber::fmt::layer())
                .init();

            // build our application with a route
            let url = format!("0.0.0.0:{}", config.api_port);

            let app = app::app(config);

            // run it
            let listener = tokio::net::TcpListener::bind(url).await.unwrap();

            tracing::info!("listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, app).await.unwrap();
        });
}

fn load_config() -> Config {
    let file_path = if cfg!(debug_assertions) {
        "api/web/config/.env"
    } else {
        "config/.env"
    };

    match std::fs::File::open(file_path) {
        Ok(f) => match serde_yaml::from_reader(f) {
            Ok(config) => config,
            Err(e) => {
                panic!("Failed to read config file: {e}")
            }
        },
        Err(_) => {
            panic!("Failed to open config file at: {file_path}")
        }
    }
}
