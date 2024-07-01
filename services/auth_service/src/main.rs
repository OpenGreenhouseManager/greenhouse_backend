use crate::schema::users::dsl::users;
use axum::{
    extract::{FromRef, State},
    routing::post,
    Json, Router,
};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl,
};
use greenhouse_core::auth_service_dto::{
    login::{LoginRequestDto, LoginResponseDto},
    register::{RegisterRequestDto, RegisterResponseDto},
};
use models::User;
use schema::users::username;
use serde::Deserialize;
pub mod models;
pub mod schema;
pub mod user_token;

#[derive(Clone, Deserialize)]
struct Config {
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
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/check_token", post(check_token))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn register(
    State(AppState { config, pool }): State<AppState>,
    Json(user): Json<RegisterRequestDto>,
) -> Json<RegisterResponseDto> {
    let mut conn = pool.get().await.unwrap();

    let new_user = User::new(
        user.username,
        user.password,
        user.role,
        config.jwt_secret.clone(),
    );

    let res = diesel::insert_into(schema::users::table)
        .values(new_user)
        .returning(User::as_returning())
        .get_result(&mut conn)
        .await
        .unwrap();
    Json(RegisterResponseDto {
        token: res.login_session,
        token_type: "Bearer".to_string(),
    })
}

#[axum::debug_handler]
async fn login(
    State(AppState { config, pool }): State<AppState>,
    Json(login): Json<LoginRequestDto>,
) -> Json<LoginResponseDto> {
    let mut conn = pool.get_owned().await.unwrap();

    let user = users
        .filter(username.eq(login.username))
        .get_result::<User>(&mut conn)
        .await
        .unwrap();

    user.check_login(login.password).await;

    Json(LoginResponseDto {
        token: user.refresh_token(config.jwt_secret),
        token_type: "Bearer".to_string(),
    })
}

#[axum::debug_handler]
async fn check_token(
    State(AppState { config, pool }): State<AppState>,
    Json(login): Json<LoginRequestDto>,
) -> Json<LoginResponseDto> {
    let mut conn = pool.get_owned().await.unwrap();

    let user = users
        .filter(username.eq(login.username))
        .get_result::<User>(&mut conn)
        .await
        .unwrap();

    user.check_login(login.password).await;

    Json(LoginResponseDto {
        token: user.refresh_token(config.jwt_secret),
        token_type: "Bearer".to_string(),
    })
}
