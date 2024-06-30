//! Run with
//!
//! ```not_rust
//! cargo run -p example-hello-world
//! ```

use axum::{
    extract::{FromRef, State},
    http::StatusCode,
    routing::get,
    Router,
};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
struct Config {
    service_port: u32,
    #[serde(rename = "DATABASE_URL")]
    database_url: String,
}

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(FromRef, Clone)]
struct AppState {
    config: Config,
    pool: Pool,
}

#[tokio::main]
async fn main() {
    let config: Config = match std::fs::File::open(".env") {
        Ok(f) => match serde_yaml::from_reader(f) {
            Ok(config) => config,
            Err(e) => {
                panic!("Failed to read config file: {}", e)
            }
        },
        Err(e) => {
            panic!("Failed to open config file: {}", e)
        }
    };

    let url = format!("localhost:{}", config.service_port);

    let pool = Pool::builder()
        .build(AsyncDieselConnectionManager::new(
            config.database_url.clone(),
        ))
        .await
        .unwrap();

    let state = AppState { config, pool };

    let app = Router::new()
        .route("/:a/:b", get(handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(State(AppState { config: _, pool: _ }): State<AppState>) -> StatusCode {
    StatusCode::OK
}
