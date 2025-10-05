use crate::AppState;
use crate::alert::service;
use crate::helper::error::HttpResult;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use greenhouse_core::data_storage_service_dto::alert_dto::alert::AlertsDto;
use greenhouse_core::data_storage_service_dto::alert_dto::get_aggrigated_alert::AgrigatedAlertsDto;
use greenhouse_core::data_storage_service_dto::alert_dto::query::{AlertQuery, IntervalQuery};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(alert_subset))
        .route("/filter", get(filter))
        .with_state(state)
}

async fn filter(
    State(AppState { config }): State<AppState>,
    Query(query): Query<AlertQuery>,
) -> HttpResult<AlertsDto> {
    let entry =
        service::get_filtered_alert(&config.service_addresses.data_storage_service, query).await?;
    Ok(entry)
}

async fn alert_subset(
    State(AppState { config }): State<AppState>,
    Query(query): Query<IntervalQuery>,
) -> HttpResult<AgrigatedAlertsDto> {
    let entry =
        service::get_alert_subset(&config.service_addresses.data_storage_service, query).await?;
    Ok(entry)
}
