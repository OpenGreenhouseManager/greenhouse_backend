use crate::{AppState, Config, Pool, router};

extern crate diesel_migrations;
use axum::Router;

use greenhouse_core::data_storage_service_dto::{
    alert_dto::endpoints::ALERT, diary_dtos::endpoints::DIARY,
};
use tower_http::trace::TraceLayer;

pub fn app(config: Config, pool: Pool) -> Router {
    let state = AppState { config, pool };

    Router::new()
        .nest(ALERT, router::alert_router::routes(state.clone()))
        .nest(DIARY, router::diary_entry_router::routes(state.clone()))
        .layer(TraceLayer::new_for_http())
}
