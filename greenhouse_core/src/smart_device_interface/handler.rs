use std::sync::Arc;

use axum::{ extract::State, http::StatusCode, Json };
use serde::{ de::DeserializeOwned, Serialize };

use crate::smart_device_dto::{
    config::{ ConfigRequestDto, ConfigResponseDto },
    read::ReadResponseDto,
    status::DeviceStatusResponseDto,
    write::WriteRequestDto,
};

use super::{ config::{ read_config_file, update_config_file }, device_service::DeviceService };

pub(crate) async fn write_device_handler<T>(
    State(device_service): State<DeviceService<T>>,
    Json(payload): Json<WriteRequestDto>
) -> StatusCode
    where T: Clone
{
    let config = device_service.config;
    match device_service.write_handler {
        Some(handler) => handler(payload.data, config),
        None => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn read_device_handler<T>(State(
    device_service,
): State<DeviceService<T>>) -> Json<ReadResponseDto>
    where T: Clone
{
    let config = device_service.config;

    match device_service.read_handler {
        None => Json(Default::default()),
        Some(handler) =>
            Json(ReadResponseDto {
                data: handler(config),
                output_type: Default::default(),
            }),
    }
}

pub(crate) async fn get_config_handler<T>(State(
    mut device_service,
): State<DeviceService<T>>) -> Json<Option<ConfigResponseDto<T>>>
    where T: DeserializeOwned + Clone
{
    match read_config_file() {
        Ok(config) => {
            device_service.config = Arc::new(config.clone());
            Json(Some(ConfigResponseDto::from(config)))
        }
        Err(_) => Json(None),
    }
}
pub(crate) async fn status_device_handler<T>(State(
    device_service,
): State<DeviceService<T>>) -> Json<DeviceStatusResponseDto>
    where T: Clone
{
    let config = device_service.config;

    Json((device_service.status_handler)(config))
}

pub(crate) async fn config_update_handler<T>(
    State(device_service): State<DeviceService<T>>,
    Json(config): Json<ConfigRequestDto<T>>
) -> StatusCode
    where T: Serialize + Clone
{
    let config = (device_service.config_interceptor_handler)(config);

    match update_config_file(&config) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
