use crate::database::schema::scripting_device;
use crate::{
    AppState,
    database::scripting::ScriptingDevice,
    router::error::{Error, HttpResult},
};
use axum::{
    Json, Router,
    extract::State,
    routing::{delete, post},
};
use diesel::ExpressionMethods;
use diesel::query_dsl::methods::FilterDsl;
use diesel_async::RunQueryDsl;
use greenhouse_core::scripting_service_dto::token::TokenDto;
use reqwest::StatusCode;
use uuid::Uuid;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(generate_scripting_key))
        .route("/", delete(delete_scripting_key))
        .route("/check", post(check_scripting_key))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn generate_scripting_key(
    State(AppState { config: _, pool }): State<AppState>,
) -> HttpResult<TokenDto> {
    let token = Uuid::new_v4().to_string();

    let device = ScriptingDevice {
        scriptig_key: token.clone(),
    };

    let mut conn = pool.get().await.map_err(|e| {
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;
    diesel::insert_into(scripting_device::table)
        .values(&device)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::Creation
        })?;

    Ok(TokenDto { token })
}

pub(crate) async fn check_scripting_key(
    State(AppState { config: _, pool }): State<AppState>,
    Json(check_token_dto_request): Json<TokenDto>,
) -> HttpResult<StatusCode> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;

    let _ = scripting_device::table
        .filter(scripting_device::scriptig_key.eq(check_token_dto_request.token))
        .first::<ScriptingDevice>(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::NotFound
        });

    Ok(StatusCode::OK)
}

pub(crate) async fn delete_scripting_key(
    State(AppState { config: _, pool }): State<AppState>,
    Json(check_token_dto_request): Json<TokenDto>,
) -> HttpResult<StatusCode> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;

    let _ = diesel::delete(scripting_device::table)
        .filter(scripting_device::scriptig_key.eq(check_token_dto_request.token))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::NotFound
        });

    Ok(StatusCode::OK)
}
