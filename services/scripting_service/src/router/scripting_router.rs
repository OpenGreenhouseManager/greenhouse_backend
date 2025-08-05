use crate::database::schema::scripting_device;
use crate::{
    AppState,
    database::scripting::ScriptingDevice,
    router::error::{Error, HttpResult},
};
use axum::routing::post;
use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use diesel::query_dsl::methods::FilterDsl;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use greenhouse_core::scripting_dto::check_token::CheckTokenDtoRequest;
use greenhouse_core::scripting_dto::get_token::GetTokenDtoResponse;
use reqwest::StatusCode;
use uuid::Uuid;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(generate_scripting_key))
        .route("/check", post(check_scripting_key))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn generate_scripting_key(
    State(AppState { config: _, pool }): State<AppState>,
) -> HttpResult<impl IntoResponse> {
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

    Ok(Json(GetTokenDtoResponse { token }))
}

pub(crate) async fn check_scripting_key(
    State(AppState { config: _, pool }): State<AppState>,
    Json(check_token_dto_request): Json<CheckTokenDtoRequest>,
) -> HttpResult<impl IntoResponse> {
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

    Ok(StatusCode::OK.into_response())
}
