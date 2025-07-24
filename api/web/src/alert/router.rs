use crate::AppState;
use crate::alert::error::HttpResult;
use crate::alert::service;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use greenhouse_core::data_storage_service_dto::alert_dto::post_create_alert::CreateAlertDto;
use greenhouse_core::data_storage_service_dto::alert_dto::query::{AlertQuery, IntervalQuery};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_alert))
        .route("/", get(alert_subset))
        .route("/filter", get(filter))
        .with_state(state)
}

async fn filter(
    State(AppState { config }): State<AppState>,
    Query(query): Query<AlertQuery>,
) -> HttpResult<impl IntoResponse> {
    let entry =
        service::get_filtered_alert(&config.service_addresses.data_storage_service, query).await?;
    Ok(Json(entry))
}

async fn alert_subset(
    State(AppState { config }): State<AppState>,
    Query(query): Query<IntervalQuery>,
) -> HttpResult<impl IntoResponse> {
    let entry =
        service::get_alert_subset(&config.service_addresses.data_storage_service, query).await?;
    Ok(Json(entry))
}

async fn create_alert(
    State(AppState { config }): State<AppState>,
    Json(alert): Json<CreateAlertDto>,
) -> HttpResult<impl IntoResponse> {
    service::create_alert(&config.service_addresses.data_storage_service, alert).await?;
    Ok(())
}
