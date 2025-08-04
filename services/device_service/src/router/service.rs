use super::error::{Error, Result};
use greenhouse_core::{
    scripting_dto,
    smart_device_dto::{activation::ActivateRequestDto, endpoints},
};

pub(crate) async fn request_device_config(device_address: &str) -> Result<String> {
    let resp = reqwest::Client::new()
        .get(device_address.to_string() + endpoints::CONFIG)
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

pub(crate) async fn request_device_status(device_address: &str) -> Result<String> {
    let resp = reqwest::Client::new()
        .get(device_address.to_string() + endpoints::STATUS)
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

pub(crate) async fn request_device_activate(
    device_address: &str,
    scripting_api: ActivateRequestDto,
) -> Result<String> {
    let resp = reqwest::Client::new()
        .post(device_address.to_string() + endpoints::ACTIVATE)
        .json(&scripting_api)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!("Error in activate to smart device: {:?}", e);

            Error::SmartDeviceNotReachable
        })?;
    resp.text().await.map_err(|e| {
        sentry::capture_error(&e);

        tracing::error!("Error in response from smart device: {:?}", e);

        Error::SmartDeviceResponse
    })
}

pub(crate) async fn request_device_token(scripting_api_address: &str) -> Result<String> {
    let resp = reqwest::Client::new()
        .get(scripting_api_address.to_string() + scripting_dto::endpoints::TOKEN)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!("Error in get to scripting api: {:?}", e);

            Error::ScriptingApiNotReachable
        })?;
    resp.text().await.map_err(|e| {
        sentry::capture_error(&e);

        tracing::error!("Error in response from scripting api: {:?}", e);

        Error::ScriptingApiResponse
    })
}
