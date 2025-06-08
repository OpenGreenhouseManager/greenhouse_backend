use super::error::{Error, Result};
use greenhouse_core::smart_device_dto::endpoints;

pub(crate) async fn request_device_config(device_address: &str) -> Result<String> {
    let resp = reqwest::Client::new()
        .get(device_address.to_string() + endpoints::CONFIG + "/")
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in get to smart device: {:?} for url {}",
                e,
                device_address
            );

            Error::SmartDeviceNotReachable
        })?;
    resp.text().await.map_err(|e| {
        sentry::capture_error(&e);

        tracing::error!("Error in response from smart device: {:?}", e);

        Error::SmartDeviceResponse
    })
}
