use crate::device::error::{Error, Result};
use greenhouse_core::device_service_dto::{
    device_router::{DeviceResponseDto, PostDeviceDtoRequest, PutDeviceDtoRequest},
    endpoints,
};
use reqwest::StatusCode;
use uuid::Uuid;

pub(crate) async fn create_device(
    base_url: &str,
    device: PostDeviceDtoRequest,
) -> Result<DeviceResponseDto> {
    let resp = reqwest::Client::new()
        .post(base_url)
        .json(&device)
        .send()
        .await?;

    if resp.status() == StatusCode::OK {
        resp.json().await.map_err(Error::RequestError)
    } else {
        Err(Error::ServiceUnavailable)
    }
}

pub(crate) async fn update_device(
    base_url: &str,
    id: Uuid,
    device: PutDeviceDtoRequest,
) -> Result<DeviceResponseDto> {
    let resp = reqwest::Client::new()
        .put(format!("{}/{}", base_url, id))
        .json(&device)
        .send()
        .await?;

    match resp.status() {
        StatusCode::OK => resp.json().await.map_err(Error::RequestError),
        StatusCode::NOT_FOUND => Err(Error::NotFound),
        _ => Err(Error::ServiceUnavailable),
    }
}

pub(crate) async fn get_device(base_url: &str, id: Uuid) -> Result<DeviceResponseDto> {
    let resp = reqwest::Client::new()
        .get(format!("{}/{}", base_url, id))
        .send()
        .await?;

    match resp.status() {
        StatusCode::OK => resp.json().await.map_err(Error::RequestError),
        StatusCode::NOT_FOUND => Err(Error::NotFound),
        _ => Err(Error::ServiceUnavailable),
    }
}

pub(crate) async fn delete_device(base_url: &str, id: Uuid) -> Result<()> {
    let resp = reqwest::Client::new()
        .delete(format!("{}/{}", base_url, id))
        .send()
        .await?;

    match resp.status() {
        StatusCode::OK => Ok(()),
        StatusCode::NOT_FOUND => Err(Error::NotFound),
        _ => Err(Error::ServiceUnavailable),
    }
}

pub(crate) async fn device_call(base_url: &str, id: Uuid, json_data: String) -> Result<String> {
    let resp = reqwest::Client::new()
        .put(format!("{}/{}/{}", base_url, id, endpoints::CALL))
        .header("Content-Type", "application/json")
        .body(json_data)
        .send()
        .await?;

    match resp.status() {
        StatusCode::OK => resp.text().await.map_err(Error::RequestError),
        StatusCode::NOT_FOUND => Err(Error::NotFound),
        _ => Err(Error::ServiceUnavailable),
    }
}

pub(crate) async fn get_device_status(base_url: &str, id: Uuid) -> Result<String> {
    let resp = reqwest::Client::new()
        .get(format!("{}/{}/{}", base_url, id, endpoints::STATUS))
        .send()
        .await?;

    match resp.status() {
        StatusCode::OK => resp.json().await.map_err(Error::RequestError),
        StatusCode::NOT_FOUND => Err(Error::NotFound),
        _ => Err(Error::ServiceUnavailable),
    }
}

pub(crate) async fn get_devices(base_url: &str) -> Result<Vec<DeviceResponseDto>> {
    let resp = reqwest::Client::new()
        .get(base_url)
        .send()
        .await?;

    if resp.status() == StatusCode::OK {
        resp.json().await.map_err(Error::RequestError)
    } else {
        Err(Error::ServiceUnavailable)
    }
}
