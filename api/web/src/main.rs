use auth::middleware::check_token;
use axum::{extract::FromRef, middleware, Router};
use serde::Deserialize;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
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
}

#[derive(FromRef, Clone)]
struct AppState {
    config: Config,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "web_api=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let file_path = if cfg!(debug_assertions) {
        "api/web/config/.env"
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

    // build our application with a route
    let url = format!("0.0.0.0:{}", config.api_port);
    let state = AppState { config };
    let app = Router::new()
        .nest("/api", test::router::routes(state.clone()))
        .layer(middleware::from_fn_with_state(state.clone(), check_token))
        .merge(auth::router::routes(state))
        .layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http());

    // run it
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
