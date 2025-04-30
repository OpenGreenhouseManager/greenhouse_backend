use crate::{AppState, auth::Result};
use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::get,
};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/test", get(api_test_handler))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn api_test_handler() -> Result<Response> {
    Ok("worked".into_response())
}
