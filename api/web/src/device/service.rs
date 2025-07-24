use greenhouse_core::device_service_dto::{
    endpoints, get_device::DeviceResponseDto, post_device::PostDeviceDtoRequest,
    put_device::PutDeviceDtoRequest,
};
use uuid::Uuid;

use crate::{
    device::{Error, Result},
    helper::error::ApiError,
};

pub(crate) async fn update_device(
    base_url: &str,
    id: Uuid,
    entry: PutDeviceDtoRequest,
) -> Result<DeviceResponseDto> {
    let resp = reqwest::Client::new()
        .put(base_url.to_string() + "/" + &id.to_string())
        .json(&entry)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in update device: {:?} with entry: {:?} for url {}",
                e,
                entry,
                base_url
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error parsing json for update device: {:?} with entry: {:?} for url {}",
                e,
                entry,
                base_url
            );

            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp.text().await.unwrap_or_default(),
    }))
}

pub(crate) async fn create_device(
    base_url: &str,
    update: PostDeviceDtoRequest,
) -> Result<DeviceResponseDto> {
    let resp = reqwest::Client::new()
        .post(base_url.to_string())
        .json(&update)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in post to device service: {:?} with entry: {:?} for url {}",
                e,
                update,
                base_url
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!("Error in json to device service: {:?} with", e);

            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp.text().await.unwrap_or_default(),
    }))
}

pub(crate) async fn get_device(base_url: &str, id: Uuid) -> Result<DeviceResponseDto> {
    let resp = reqwest::Client::new()
        .get(base_url.to_string() + "/" + &id.to_string())
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in get to service: {:?} with id: {:?} for url {}",
                e,
                id,
                base_url
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!("Error in get to service: {:?} with id: {:?}", e, id);

            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp.text().await.unwrap_or_default(),
    }))
}

pub(crate) async fn get_device_config(base_ulr: &str, id: Uuid) -> Result<String> {
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string() + "/" + &id.to_string() + "/" + endpoints::CONFIG)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in get to device service: {:?} with id: {:?} for url {}",
                e,
                id,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::capture_error(&e);
            tracing::error!("Error in get to device service: {:?}", e,);
            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp.text().await.unwrap_or_default(),
    }))
}

pub(crate) async fn get_device_status(base_ulr: &str, id: Uuid) -> Result<String> {
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string() + "/" + &id.to_string() + "/" + endpoints::STATUS)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in get to service: {:?} with id: {:?} for url {}",
                e,
                id,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::capture_error(&e);
            tracing::error!("Error in get to service: {:?}", e,);
            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp.text().await.unwrap_or_default(),
    }))
}

pub(crate) async fn get_devices(base_url: &str) -> Result<Vec<DeviceResponseDto>> {
    let resp = reqwest::Client::new()
        .get(base_url.to_string())
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in get all to device service: {:?} for url {}",
                e,
                base_url
            );

            Error::Request(e)
        })?;

    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::capture_error(&e);
            tracing::error!("Error in get all json to service: {:?}", e,);
            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp.text().await.unwrap_or_default(),
    }))
}
