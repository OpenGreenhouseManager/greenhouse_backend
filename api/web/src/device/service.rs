use greenhouse_core::device_service_dto::{
    endpoints, get_device::DeviceResponseDto, post_device::PostDeviceDtoRequest,
    put_device::PutDeviceDtoRequest,
};
use uuid::Uuid;

use crate::device::{Error, Result};

pub(crate) async fn update_device(
    base_ulr: &str,
    id: Uuid,
    entry: PutDeviceDtoRequest,
) -> Result<DeviceResponseDto> {
    let resp = reqwest::Client::new()
        .put(base_ulr.to_string()  + "/" + &id.to_string())
        .json(&entry)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in update device: {:?} with entry: {:?} for url {}",
                e,
                entry,
                base_ulr
            );

            Error::InternalError
        })?;
    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);

        tracing::error!(
            "Error parsing json for update device: {:?} with entry: {:?} for url {}",
            e,
            entry,
            base_ulr
        );

        Error::InternalError
    })
}

pub(crate) async fn create_device(
    base_ulr: &str,
    update: PostDeviceDtoRequest,
) -> Result<DeviceResponseDto> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() )
        .json(&update)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in post to device service: {:?} with entry: {:?} for url {}",
                e,
                update,
                base_ulr
            );

            Error::InternalError
        })?;
    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);

        tracing::error!("Error in json to device service: {:?} with", e);

        Error::InternalError
    })
}

pub(crate) async fn get_device(base_ulr: &str, id: Uuid) -> Result<DeviceResponseDto> {
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string()  + "/" + &id.to_string())
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

            Error::InternalError
        })?;
    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);

        tracing::error!("Error in get to service: {:?} with id: {:?}", e, id);

        Error::InternalError
    })
}

pub(crate) async fn get_device_config(base_ulr: &str, id: Uuid) -> Result<String> {
    let resp = reqwest::Client::new()
        .get(
            base_ulr.to_string()
                
                + "/"
                + &id.to_string()
                + "/"
                + endpoints::CONFIG,
        )
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

            Error::InternalError
        })?;
    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);
        tracing::error!("Error in get to device service: {:?}", e,);
        Error::InternalError
    })
}

pub(crate) async fn get_device_status(base_ulr: &str, id: Uuid) -> Result<String> {
    let resp = reqwest::Client::new()
        .get(
            base_ulr.to_string()
                
                + "/"
                + &id.to_string()
                + "/"
                + endpoints::STATUS,
        )
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

            Error::InternalError
        })?;
    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);
        tracing::error!("Error in get to service: {:?}", e,);
        Error::InternalError
    })
}

pub(crate) async fn get_devices(base_ulr: &str) -> Result<Vec<DeviceResponseDto>> {
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string())
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in get all to device service: {:?} for url {}",
                e,
                base_ulr
            );

            Error::InternalError
        })?;

    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);
        tracing::error!("Error in get all json to service: {:?}", e,);
        Error::InternalError
    })
}
