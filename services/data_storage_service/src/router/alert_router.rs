use super::error::Result;
use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::AlertDto,
    get_aggrigated_alert::AlertAggrigatedDto,
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
) -> Result<impl IntoResponse> {
    let a: Vec<AlertDto> = Alert::query(query, &pool)
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
