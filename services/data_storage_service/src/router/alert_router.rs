use super::error::HttpResult;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::{AlertDto, AlertsDto},
    get_aggrigated_alert::{AggrigatedAlertDto, AgrigatedAlertsDto},
    post_create_alert::CreateAlertDto,
    query::{AlertQuery, IntervalQuery},
};

use crate::{AppState, database::alert_models::Alert};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_alert))
        .route("/", get(alert_subset))
        .route("/filter", get(filter))
        .with_state(state)
}

async fn filter(
    State(AppState { config: _, pool }): State<AppState>,
    Query(query): Query<AlertQuery>,
) -> HttpResult<AlertsDto> {
    let a = Alert::query(query, &pool)
        .await?
        .into_iter()
        .map(|a| a.into())
        .collect::<Vec<AlertDto>>();
    Ok(a.into())
}

async fn alert_subset(
    State(AppState { config: _, pool }): State<AppState>,
    Query(query): Query<IntervalQuery>,
) -> HttpResult<AgrigatedAlertsDto> {
    let a = Alert::aggrigate(query, &pool)
        .await?
        .into_iter()
        .map(|a| a.into())
        .collect::<Vec<AggrigatedAlertDto>>()
        .into();
    Ok(a)
}

async fn create_alert(
    State(AppState { config: _, pool }): State<AppState>,
    Json(alert): Json<CreateAlertDto>,
) -> HttpResult<AlertDto> {
    let alert = Alert::create(alert, &pool).await?.into();
    Ok(alert)
}
