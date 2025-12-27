use crate::{AppState, helper::error::HttpResult};
use axum::{Json, Router};
use axum::extract::State;
use axum::routing::post;
use greenhouse_core::notification_service_dto::push::PushSubscriptionDto;

use super::service;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/subscribe", post(subscribe))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn subscribe(
    State(AppState { config }): State<AppState>,
    Json(entry): Json<PushSubscriptionDto>,
) -> HttpResult<()> {
    service::subscribe(&config.service_addresses.notification_service, entry).await?;
    Ok(())
}


