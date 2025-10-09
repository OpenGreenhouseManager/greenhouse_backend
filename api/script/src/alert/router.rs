use crate::{AppState, alert::service, helper::error::HttpResult};
use axum::{Json, Router, extract::State, routing::post};
use greenhouse_core::data_storage_service_dto::alert_dto::post_create_alert::CreateAlertDto;
use reqwest::StatusCode;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_alert))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn create_alert(
    State(AppState { config }): State<AppState>,
    Json(alert): Json<CreateAlertDto>,
) -> HttpResult<StatusCode> {
    service::create_alert(&config.service_addresses.data_storage_service, alert).await?;
    Ok(StatusCode::OK)
}
