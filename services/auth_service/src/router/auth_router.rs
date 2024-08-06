use super::{Error, Result};
use crate::{database::schema::users::dsl::users, user_token::UserToken};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use database::schema::users::{id, login_session, username};
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods};

use diesel_async::RunQueryDsl;
use greenhouse_core::auth_service_dto::{
    login::{LoginRequestDto, LoginResponseDto},
    register::{RegisterRequestDto, RegisterResponseDto},
    token::TokenRequestDto,
};

use crate::{
    database::{self, models::User},
    AppState,
};

#[axum::debug_handler]
pub async fn register(
    State(AppState { config, pool }): State<AppState>,
    Json(user): Json<RegisterRequestDto>,
) -> Result<Response> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::configure_scope(|scope| {
            let mut map = std::collections::BTreeMap::new();
            map.insert(String::from("db_url"), config.database_url.into());

            scope.set_context("db", sentry::protocol::Context::Other(map));
        });
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;
    let role = "user";
    let mut new_user = User::new(&user.username, &user.password, role)?;
    let token = new_user.refresh_token(&config.jwt_secret)?;
    let _ = diesel::insert_into(database::schema::users::table)
        .values(new_user)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(user.username.clone()),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), user.username.into());
                map.insert(String::from("role"), role.into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);
            Error::UsernameTaken
        })?;
    Ok(Json(RegisterResponseDto {
        token,
        token_type: String::from("Bearer"),
    })
    .into_response())
}

#[axum::debug_handler]
pub async fn login(
    State(AppState { config, pool }): State<AppState>,
    Json(login): Json<LoginRequestDto>,
) -> Result<Response> {
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
            Error::DatabaseConnection
        })?;

    if !user.check_login(&login.password).await? {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
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
pub async fn check_token(
    State(AppState { config, pool }): State<AppState>,
    Json(token): Json<TokenRequestDto>,
) -> Result<Response> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::configure_scope(|scope| {
            let mut map = std::collections::BTreeMap::new();
            map.insert(String::from("db_url"), config.database_url.into());

            scope.set_context("db", sentry::protocol::Context::Other(map));
        });
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;

    let claims = UserToken::get_claims(&token.token, &config.jwt_secret)?;

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
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    Ok(StatusCode::OK.into_response())
}
