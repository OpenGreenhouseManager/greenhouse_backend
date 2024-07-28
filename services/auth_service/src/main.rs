pub use self::error::{Error, Result};
use crate::database::schema::users::dsl::users;
use axum::{
    extract::{FromRef, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use core::panic;
use database::models::User;
use database::schema::users::{id, login_session, username};
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods};
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl,
};
use greenhouse_core::auth_service_dto::{
    login::{LoginRequestDto, LoginResponseDto},
    register::{RegisterRequestDto, RegisterResponseDto},
    token::TokenRequestDto,
};
use serde::Deserialize;
use user_token::UserToken;

pub mod database;
mod error;
pub mod user_token;

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
    let config: Config = match std::fs::File::open("config/.env") {
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

    let url = format!("0.0.0.0:{}", config.service_port);

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
) -> Result<Response> {
    let mut conn = pool.get().await.map_err(|_| Error::DatabaseConnection)?;

    let mut new_user = User::new(user.username, user.password, "user".to_string())?;
    let token = new_user.refresh_token(config.jwt_secret.clone())?;
    let _ = diesel::insert_into(database::schema::users::table)
        .values(new_user)
        .execute(&mut conn)
        .await
        .map_err(|_| Error::DatabaseConnection)?;
    Ok(Json(RegisterResponseDto {
        token,
        token_type: "Bearer".to_string(),
    })
    .into_response())
}

#[axum::debug_handler]
async fn login(
    State(AppState { config, pool }): State<AppState>,
    Json(login): Json<LoginRequestDto>,
) -> Result<Response> {
    let mut conn = pool.get().await.map_err(|_| Error::DatabaseConnection)?;

    let mut user = users
        .filter(username.eq(login.username))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|_| Error::DatabaseConnection)?;

    if user.check_login(login.password).await? {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let token = user.refresh_token(config.jwt_secret.clone())?;

    diesel::update(users)
        .filter(id.eq(user.id))
        .set(login_session.eq(token.clone()))
        .execute(&mut conn)
        .await
        .map_err(|_| Error::DatabaseConnection)?;

    Ok(Json(LoginResponseDto {
        token,
        token_type: "Bearer".to_string(),
    })
    .into_response())
}

#[axum::debug_handler]
async fn check_token(
    State(AppState { config, pool }): State<AppState>,
    Json(token): Json<TokenRequestDto>,
) -> Result<Response> {
    let mut conn = pool.get().await.map_err(|_| Error::DatabaseConnection)?;

    let claims = UserToken::get_claims(token.token.clone(), config.jwt_secret)?;

    let user = users
        .filter(username.eq(claims.user_name))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|_| Error::DatabaseConnection)?;

    if token.token != user.login_session {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    Ok(StatusCode::OK.into_response())
}
