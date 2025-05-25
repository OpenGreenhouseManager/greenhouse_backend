use crate::{
    AppState, Config, Pool,
    router::auth_router::{check_token, generate_one_time_token, login, register, register_admin},
};

extern crate diesel_migrations;
use axum::{Router, routing::post};
use greenhouse_core::auth_service_dto::endpoints;
use tower_http::trace::TraceLayer;

pub fn app(config: Config, pool: Pool) -> Router {
    let state = AppState { config, pool };

    Router::new()
        .route(endpoints::REGISTER, post(register))
        .route(endpoints::LOGIN, post(login))
        .route(endpoints::CHECK_TOKEN, post(check_token))
        .route(endpoints::ADMIN_REGISTER, post(register_admin))
        .route(
            endpoints::GENERATE_ONE_TIME_TOKEN,
            post(generate_one_time_token),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
