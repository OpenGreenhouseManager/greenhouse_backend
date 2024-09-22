use super::error::Result;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use serde::Deserialize;

use crate::{
    database::alert_models::{Alert, AlertQuery},
    AppState,
};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(alert_subset))
        .route("/filter", get(filter))
        .with_state(state)
}

async fn filter(
    State(AppState { config: _, pool }): State<AppState>,
    query: Query<AlertQuery>,
) -> Result<impl IntoResponse> {
    Alert::query(query, &pool).await
}

async fn alert_subset(
    State(AppState { config: _, pool }): State<AppState>,
) -> Result<impl IntoResponse> {
    Alert::aggrigate(&pool).await
}
