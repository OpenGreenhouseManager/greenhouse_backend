use auth::middleware::check_token;
use axum::{extract::FromRef, middleware, Router};
use reqwest::{header::{ACCEPT, AUTHORIZATION}, Method};
use serde::Deserialize;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
pub mod auth;
pub mod test;

#[derive(Clone, Deserialize)]
struct ServiceAddresses {
    #[serde(rename = "AUTH_SERVICE")]
    auth_service: String,
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
            let state = AppState { config };

            let cors = CorsLayer::new()

                .allow_headers([AUTHORIZATION, ACCEPT, reqwest::header::CONTENT_TYPE])
                // allow any headers
                .allow_credentials(true)
                // allow `POST` when accessing the resource
                .allow_methods([Method::POST])
                // allow requests from below origins
                .allow_origin(["http://localhost:4200".parse().unwrap(), "https://localhost:5001".parse().unwrap()]);


            let app = Router::new()
                .nest("/api", test::router::routes(state.clone()))
                .layer(middleware::from_fn_with_state(state.clone(), check_token))
                .merge(auth::router::routes(state))
                .layer(CookieManagerLayer::new())
                .layer(cors)
                .layer(TraceLayer::new_for_http());

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
                panic!("Failed to read config file: {}", e)
            }
        },
        Err(_) => {
            panic!("Failed to open config file at: {}", file_path)
        }
    }
}
