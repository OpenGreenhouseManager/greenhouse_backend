use greenhouse_core::{
    http_error::ErrorResponseBody,
    notification_service_dto::{endpoints, push::PushSubscriptionDto},
};

use crate::{
    helper::error::ApiError,
    push::{Error, Result},
};

pub(crate) async fn subscribe(base_url: &str, entry: PushSubscriptionDto) -> Result<()> {
    let resp = reqwest::Client::new()
        .post(base_url.to_string() + "/push/" + endpoints::SUBSCRIBE)
        .json(&entry)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            tracing::error!(
                "Error in post to notification service: {:?} with entry: {:?} for url {}",
                e,
                entry,
                base_url
            );
            Error::Request(e)
        })?;

    if resp.status().is_success() {
        return Ok(());
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp
            .json::<ErrorResponseBody>()
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                tracing::error!("Error parsing error from notification service: {:?}", e);
                Error::Json(e)
            })?
            .error,
    }))
}


