use super::error::Result;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::AlertDto, get_aggrigated_alert::AlertAggrigatedDto, post_create_alert::CreateAlertDto,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{alert_models::Alert, severity_models::Severity},
    AppState,
};

#[derive(Deserialize)]
pub struct AlertQuery {
    pub severity: Option<Severity>,
    pub identifier: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub datasource_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct IntervalQuery {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_alert))
        .route("/", get(alert_subset))
        .route("/filter", get(filter))
        .with_state(state)
}

async fn filter(
    State(AppState { config: _, pool }): State<AppState>,
    query: Query<AlertQuery>,
) -> Result<impl IntoResponse> {
    let a: Vec<AlertDto> = Alert::query(query.0, &pool)
        .await?
        .into_iter()
        .map(|a| a.into())
        .collect();
    Ok(Json(a))
}

async fn alert_subset(
    State(AppState { config: _, pool }): State<AppState>,
    Query(query): Query<IntervalQuery>,
) -> Result<impl IntoResponse> {
    let a: Vec<AlertAggrigatedDto> = Alert::aggrigate(query, &pool)
        .await?
        .into_iter()
        .map(|a| a.into())
        .collect();
    Ok(Json(a))
}

async fn create_alert(
    State(AppState { config: _, pool }): State<AppState>,
    Json(alert): Json<CreateAlertDto>,
) -> Result<impl IntoResponse> {
    let alert: AlertDto = Alert::create(alert, &pool).await?.into();
    Ok(Json(alert))
}
