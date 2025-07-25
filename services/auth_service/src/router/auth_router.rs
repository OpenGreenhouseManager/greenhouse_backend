use super::{Error, HttpResult};
use crate::token;
use crate::{
    AppState,
    database::{self, models::User},
};
use crate::{
    Config, Pool, database::schema::users::dsl::users, token::one_time_token::check_one_time_token,
};
use axum::response::IntoResponse;
use axum::{Json, extract::State, http::StatusCode, response::Response};
use database::schema::users::{id, login_session, username};
use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use diesel_async::RunQueryDsl;
use greenhouse_core::auth_service_dto::generate_one_time_token::{
    GenerateOneTimeTokenRequestDto, GenerateOneTimeTokenResponseDto,
};
use greenhouse_core::auth_service_dto::{
    login::{LoginRequestDto, LoginResponseDto},
    register::{RegisterRequestDto, RegisterResponseDto},
    register_admin::{RegisterAdminRequestDto, RegisterAdminResponseDto},
    token::TokenRequestDto,
};

#[axum::debug_handler]
pub(crate) async fn register(
    State(AppState { config, pool }): State<AppState>,
    Json(user): Json<RegisterRequestDto>,
) -> HttpResult<Response> {
    let role = "user";
    check_one_time_token(
        &user.username,
        user.one_time_token
            .parse::<u64>()
            .map_err(|_| Error::OneTimeToken)?,
        &config.jwt_secret,
    )?;
    let token = register_user(&user.username, &user.password, role, &config, &pool).await?;
    Ok(Json(RegisterResponseDto {
        token,
        token_type: String::from("Bearer"),
    })
    .into_response())
}

#[axum::debug_handler]
pub(crate) async fn register_admin(
    State(AppState { config, pool }): State<AppState>,
    Json(user): Json<RegisterAdminRequestDto>,
) -> HttpResult<Response> {
    let role = "admin";
    let token = register_user(&user.username, &user.password, role, &config, &pool).await?;
    Ok(Json(RegisterAdminResponseDto {
        token,
        token_type: String::from("Bearer"),
    })
    .into_response())
}

#[axum::debug_handler]
pub(crate) async fn generate_one_time_token(
    State(AppState { config, pool: _ }): State<AppState>,
    Json(user): Json<GenerateOneTimeTokenRequestDto>,
) -> HttpResult<Response> {
    let token =
        crate::token::one_time_token::generate_one_time_token(&user.username, &config.jwt_secret);
    Ok(Json(GenerateOneTimeTokenResponseDto {
        token: token.to_string(),
    })
    .into_response())
}

pub(crate) async fn register_user(
    name: &str,
    password: &str,
    role: &str,
    config: &Config,
    pool: &Pool,
) -> HttpResult<String> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::configure_scope(|scope| {
            let mut map = std::collections::BTreeMap::new();
            map.insert(String::from("db_url"), config.database_url.clone().into());

            scope.set_context("db", sentry::protocol::Context::Other(map));
        });
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;

    let mut new_user = User::new(name, password, role)?;
    let token = new_user.refresh_token(&config.jwt_secret)?;
    let _ = diesel::insert_into(database::schema::users::table)
        .values(new_user)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(String::from(name)),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), name.into());
                map.insert(String::from("role"), role.into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);
            Error::UsernameTaken
        })?;
    Ok(token)
}

#[axum::debug_handler]
pub(crate) async fn login(
    State(AppState { config, pool }): State<AppState>,
    Json(login): Json<LoginRequestDto>,
) -> HttpResult<Response> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::configure_scope(|scope| {
            let mut map = std::collections::BTreeMap::new();
            map.insert(String::from("db_url"), config.database_url.into());

            scope.set_context("db", sentry::protocol::Context::Other(map));
        });
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;

    let mut user = users
        .filter(username.eq(&login.username))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(login.username.clone()),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), login.username.clone().into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);
            match e {
                diesel::result::Error::NotFound => Error::UserNotFound,
                _ => Error::DatabaseConnection,
            }
        })?;

    if !user.check_login(&login.password).await? {
        return Err(Error::PasswordIncorrect.into());
    }

    let token = user.refresh_token(&config.jwt_secret)?;

    diesel::update(users)
        .filter(id.eq(user.id))
        .set(login_session.eq(&token))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(login.username.clone()),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), login.username.into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);
            Error::UserNotFound
        })?;

    Ok(Json(LoginResponseDto {
        token,
        token_type: String::from("Bearer"),
    })
    .into_response())
}

#[axum::debug_handler]
pub(crate) async fn check_token(
    State(AppState { config, pool }): State<AppState>,
    Json(token): Json<TokenRequestDto>,
) -> HttpResult<Response> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::configure_scope(|scope| {
            let mut map = std::collections::BTreeMap::new();
            map.insert(String::from("db_url"), config.database_url.into());

            scope.set_context("db", sentry::protocol::Context::Other(map));
        });
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;

    let claims = token::user_token::get_claims(&token.token, &config.jwt_secret)?;

    let user = users
        .filter(username.eq(&claims.user_name))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(claims.user_name.clone()),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), claims.user_name.clone().into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);
            Error::UserNotFound
        })?;

    if token.token != user.login_session {
        return Err(Error::TokenInvalid.into());
    }

    Ok(StatusCode::OK.into_response())
}
