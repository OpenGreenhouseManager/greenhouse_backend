use super::{Error, HttpResult};
use crate::database::schema::preferences::{alert_preferences, dashboard_preferences, user_id};
use crate::token;
use crate::{
    AppState,
    database::{self, models::Preferences, models::User},
};
use crate::{
    Config, Pool, database::schema::preferences::dsl::preferences,
    database::schema::users::dsl::users, token::one_time_token::check_one_time_token,
};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{Json, extract::State, response::Response};
use database::schema::users::{id, login_session, username};
use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use diesel_async::RunQueryDsl;
use greenhouse_core::auth_service_dto::generate_one_time_token::{
    GenerateOneTimeTokenRequestDto, GenerateOneTimeTokenResponseDto,
};
use greenhouse_core::auth_service_dto::token::TokenResponseDto;
use greenhouse_core::auth_service_dto::user_preferences::{
    SetPreferencesRequestDto, UserPreferencesRequestDto, UserPreferencesResponseDto,
};
use greenhouse_core::auth_service_dto::{
    login::{LoginRequestDto, LoginResponseDto},
    register::{RegisterRequestDto, RegisterResponseDto},
    register_admin::{RegisterAdminRequestDto, RegisterAdminResponseDto},
    token::TokenRequestDto,
};
use uuid::Uuid;

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
pub(crate) async fn register_guest(
    State(AppState { config, pool }): State<AppState>,
    Json(user): Json<RegisterAdminRequestDto>,
) -> HttpResult<Response> {
    let role = "guest";
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

    Ok(Json(TokenResponseDto { role: user.role }).into_response())
}

#[axum::debug_handler]
pub(crate) async fn get_preferences(
    State(AppState { config, pool }): State<AppState>,
    Json(token_request): Json<TokenRequestDto>,
) -> HttpResult<UserPreferencesResponseDto> {
    let mut conn = pool.get().await.map_err(|_| Error::DatabaseConnection)?;

    // Extract user info from token
    let claims = token::user_token::get_claims(&token_request.token, &config.jwt_secret)?;

    let user = users
        .filter(username.eq(&claims.user_name))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|_| Error::UserNotFound)?;

    if token_request.token != user.login_session {
        return Err(Error::TokenInvalid.into());
    }

    let pref = preferences
        .filter(user_id.eq(&user.id))
        .get_result::<Preferences>(&mut conn)
        .await
        .map_err(|_| Error::DatabaseConnection)?;
    Ok(UserPreferencesResponseDto {
        dashboard_preferences: pref.dashboard_preferences,
        alert_preferences: pref.alert_preferences,
    })
}

#[axum::debug_handler]
pub(crate) async fn set_preferences(
    State(AppState { config, pool }): State<AppState>,
    Json(request): Json<SetPreferencesRequestDto>,
) -> HttpResult<UserPreferencesResponseDto> {
    let mut conn = pool.get().await.map_err(|e| {
        tracing::error!("Error getting database connection: {:?}", e);
        Error::DatabaseConnection
    })?;

    // Extract user info from token
    let claims = token::user_token::get_claims(&request.token, &config.jwt_secret)?;

    let user = users
        .filter(username.eq(&claims.user_name))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|_| Error::UserNotFound)?;

    if request.token != user.login_session {
        return Err(Error::TokenInvalid.into());
    }

    diesel::insert_into(preferences)
        .values(Preferences::new(
            user.id,
            request.preferences.dashboard_preferences.clone(),
            request.preferences.alert_preferences.clone(),
        ))
        .on_conflict(user_id)
        .do_update()
        .set((
            dashboard_preferences.eq(request.preferences.dashboard_preferences.clone()),
            alert_preferences.eq(request.preferences.alert_preferences.clone()),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e: diesel::result::Error| {
            tracing::error!("Error setting preferences: {:?}", e);
            Error::DatabaseConnection
        })?;

    Ok(UserPreferencesResponseDto {
        dashboard_preferences: request.preferences.dashboard_preferences,
        alert_preferences: request.preferences.alert_preferences,
    })
}
