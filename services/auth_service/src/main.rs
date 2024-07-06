use core::panic;

use crate::database::schema::users::dsl::users;
use axum::{
    extract::{FromRef, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};

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
) -> Response {
    let mut conn = pool.get().await.unwrap();

    let mut new_user = match User::new(user.username, user.password, user.role) {
        Ok(user) => user,
        Err(_) => panic!("Failed to create user"),
    };
    let token = new_user.refresh_token(config.jwt_secret.clone());
    let _ = diesel::insert_into(database::schema::users::table)
        .values(new_user)
        .execute(&mut conn)
        .await
        .unwrap();
    Json(RegisterResponseDto {
        token,
        token_type: "Bearer".to_string(),
    })
    .into_response()
}

#[axum::debug_handler]
async fn login(
    State(AppState { config, pool }): State<AppState>,
    Json(login): Json<LoginRequestDto>,
) -> Response {
    let mut conn = pool.get_owned().await.unwrap();

    let mut user = users
        .filter(username.eq(login.username))
        .get_result::<User>(&mut conn)
        .await
        .unwrap();

    match user.check_login(login.password).await {
        Ok(true) => {}
        Ok(false) => {
            return Json(LoginResponseDto {
                token: "".to_string(),
                token_type: "".to_string(),
            })
            .into_response();
        }
        Err(_) => panic!("Failed to check login"),
    };

    let token = user.refresh_token(config.jwt_secret.clone());

    diesel::update(users)
        .filter(id.eq(user.id))
        .set(login_session.eq(token.clone()))
        .execute(&mut conn)
        .await
        .unwrap();

    Json(LoginResponseDto {
        token,
        token_type: "Bearer".to_string(),
    })
    .into_response()
}

#[axum::debug_handler]
async fn check_token(
    State(AppState { config, pool }): State<AppState>,
    Json(token): Json<TokenRequestDto>,
) -> Response {
    let mut conn = pool.get_owned().await.unwrap();

    let claims = UserToken::get_claims(token.token.clone(), config.jwt_secret);

    let user = users
        .filter(username.eq(claims.user_name))
        .get_result::<User>(&mut conn)
        .await
        .unwrap();

    if token.token != user.login_session {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    StatusCode::OK.into_response()
}
