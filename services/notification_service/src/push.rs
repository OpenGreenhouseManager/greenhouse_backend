use std::fs::File;
use std::io::Read;

use crate::{AppState, Config};
use crate::database::models::PushSubscription;
use greenhouse_core::notification_service_dto::push::PushSubscriptionDto;
use web_push::{ContentEncoding, IsahcWebPushClient, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder, WebPushClient, WebPushError, WebPushMessageBuilder};

pub(crate) fn build_payload_json() -> String {
	serde_json::json!({
		"notification": {
			"title": "Greenhouse alert (test)",
			"body": "This is a local test notification",
			"data": { "url": "/alerts" }
		}
	}).to_string()
}

fn to_subscription_info(endpoint: &str, p256dh: &str, auth: &str) -> SubscriptionInfo {
	SubscriptionInfo {
		endpoint: endpoint.to_string(),
		keys: SubscriptionKeys {
			p256dh: p256dh.to_string(),
			auth: auth.to_string(),
		},
	}
}

pub(crate) async fn send_to_subscription(
	config: &Config,
	sub: &PushSubscription,
	payload: &str,
) -> Result<(), WebPushError> {
	tracing::info!("Sending web push to subscription: {:?}", sub.endpoint);
	let subscription_info = to_subscription_info(&sub.endpoint, &sub.p256dh, &sub.auth);
	let file = File::open("/home/maudi/.ssh/vipid/private_key.pem").unwrap();
	let mut sig_builder = VapidSignatureBuilder::from_pem(file, &subscription_info)?;
	let client = IsahcWebPushClient::new()?;
	tracing::info!("created signature builder");
	// Subject is recommended (mailto or website)
	sig_builder.add_claim("sub", config.vapid_subject.clone());
	let signature = sig_builder.build()?;
	tracing::info!("created signature");
	let mut builder = WebPushMessageBuilder::new(&subscription_info);
	builder.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());
	builder.set_vapid_signature(signature);

	let message = builder.build()?;
	client.send(message).await.map(|_| ())
}

pub(crate) async fn broadcast_to_all(state: &AppState) {
	let payload = build_payload_json();
	match PushSubscription::all(&state.pool).await {
		Ok(subs) => {
			for sub in subs {
				if let Err(e) = send_to_subscription(&state.config, &sub, &payload).await {
					tracing::warn!("Failed to send web push to {}: {:?}", sub.endpoint, e);
				}
			}
		}
		Err(e) => {
			tracing::error!("Failed to load subscriptions for broadcast: {}", e);
		}
	}
}


