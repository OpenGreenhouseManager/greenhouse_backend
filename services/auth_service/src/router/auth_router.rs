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
    let mut conn = pool.get().await.map_err(|_| Error::DatabaseConnection)?;

    let mut new_user = User::new(&user.username, &user.password, "user")?;
    let token = new_user.refresh_token(&config.jwt_secret)?;
    let _ = diesel::insert_into(database::schema::users::table)
        .values(new_user)
        .execute(&mut conn)
        .await
        .map_err(|_| Error::UsernameTaken)?;
    Ok(Json(RegisterResponseDto {
        token,
        token_type: "Bearer".to_string(),
    })
    .into_response())
}

#[axum::debug_handler]
pub async fn login(
    State(AppState { config, pool }): State<AppState>,
    Json(login): Json<LoginRequestDto>,
) -> Result<Response> {
    let mut conn = pool.get().await.map_err(|_| Error::DatabaseConnection)?;

    let mut user = users
        .filter(username.eq(login.username))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|_| Error::DatabaseConnection)?;

    if !user.check_login(&login.password).await? {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let token = user.refresh_token(&config.jwt_secret)?;

    diesel::update(users)
        .filter(id.eq(user.id))
        .set(login_session.eq(&token))
        .execute(&mut conn)
        .await
        .map_err(|_| Error::UserNotFound)?;

    Ok(Json(LoginResponseDto {
        token,
        token_type: "Bearer".to_string(),
    })
    .into_response())
}

#[axum::debug_handler]
pub async fn check_token(
    State(AppState { config, pool }): State<AppState>,
    Json(token): Json<TokenRequestDto>,
) -> Result<Response> {
    let mut conn = pool.get().await.map_err(|_| Error::DatabaseConnection)?;

    let claims = UserToken::get_claims(&token.token, &config.jwt_secret)?;

    let user = users
        .filter(username.eq(claims.user_name))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|_| Error::UserNotFound)?;

    if token.token != user.login_session {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    Ok(StatusCode::OK.into_response())
}
