use axum::Router;

use crate::AppState;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new().with_state(state)
}
