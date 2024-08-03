use crate::{auth::Result, AppState};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/test", get(api_test_handler))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn api_test_handler(
    State(AppState { config }): State<AppState>,
) -> Result<Response> {
    Ok("worked".into_response())
}
