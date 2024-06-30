//! Run with
//!
//! ```not_rust
//! cargo run -p example-hello-world
//! ```

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
struct Config {
    service_port: u32,
    database_name: String,
    database_port: u32,
    database_user: String,
    database_password: String,
}

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(Clone)]
struct AppState {
    config: Config,
    pool: Pool,
}

#[tokio::main]
async fn main() {
    let config: Config = match std::fs::File::open("config.yaml") {
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

    let connection_string = format!(
        "postgres://{}:{}@localhost:{}/{}",
        config.database_user, config.database_password, config.database_port, config.database_name
    );
    let url = format!("localhost:{}", config.service_port);

    let state = AppState {
        config,
        pool: Pool::builder()
            .build(AsyncDieselConnectionManager::new(connection_string))
            .await
            .unwrap(),
    };

    let app = Router::new()
        .route("/:a/:b", get(handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(State(pool): State<Pool>) -> StatusCode {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR));

    let res = diesel::insert_into(users::table)
        .values(new_user)
        .returning(User::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(internal_error)?;
    Ok(Json(res))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
