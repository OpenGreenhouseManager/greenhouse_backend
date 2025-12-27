use crate::AppState;
use crate::database::models::PushSubscription;
use crate::push;
use axum::{routing::post, Json, Router};
use axum::extract::State;
use greenhouse_core::notification_service_dto::push::PushSubscriptionDto;

use super::error::{Error, HttpResult};

pub(crate) fn routes(state: AppState) -> Router {
	Router::new()
		.route("/subscribe", post(subscribe))
		.route("/internal/broadcast", post(internal_broadcast))
		.with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn subscribe(
	State(AppState { pool, .. }): State<AppState>,
	Json(entry): Json<PushSubscriptionDto>,
) -> HttpResult<()> {
	tracing::info!("Received push subscription: {:?}", entry);

	let endpoint = entry.endpoint;
	let p256dh = entry.keys.p256dh;
	let auth = entry.keys.auth;

	match PushSubscription::find_by_endpoint(&endpoint, &pool).await {
		Ok(Some(mut existing)) => {
			existing.update_keys(p256dh, auth, &pool).await.map_err(|e| {
				sentry::capture_error(&e);
				Error::DatabaseConnection
			})?;
		}
		Ok(None) => {
			let new = PushSubscription::new(endpoint, p256dh, auth);
			new.flush(&pool).await.map_err(|e| {
				sentry::capture_error(&e);
				Error::DatabaseConnection
			})?;
		}
		Err(e) => {
			sentry::capture_error(&e);
			return Err(Error::DatabaseConnection.into());
		}
	}

	Ok(())
}

#[axum::debug_handler]
pub(crate) async fn internal_broadcast(
	State(state): State<AppState>,
) -> HttpResult<()> {
	tracing::info!("Broadcast requested");
	let payload = push::build_payload_json();
	// Fire-and-forget broadcast; errors are logged inside
	push::broadcast_to_all(&state).await;
	tracing::info!("Broadcast requested with payload: {}", payload);
	Ok(())
}

