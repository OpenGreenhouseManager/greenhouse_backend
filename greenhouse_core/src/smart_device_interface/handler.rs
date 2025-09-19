use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    smart_device_dto::{
        Type,
        activation::ActivateRequestDto,
        config::{ConfigRequestDto, ConfigResponseDto},
        read::ReadResponseDto,
        status::DeviceStatusResponseDto,
        write::WriteRequestDto,
    },
    smart_device_interface::config::{Config, ScriptingApi},
};

use super::{
    config::{read_config_file_with_path, update_config_file_with_path},
    device_builder::DeviceBuilder,
};

pub(crate) async fn write_device_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
    Json(payload): Json<WriteRequestDto>,
) -> StatusCode
where
    T: Clone + Default + DeserializeOwned,
{
    let config = device_service
        .config
        .read()
        .map(|c| c.clone())
        .unwrap_or_else(|_| Arc::new(Config::<T>::default()));

    match device_service.write_handler {
        Some(handler) => handler(payload.data, config).await,
        None => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn read_device_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
) -> Json<ReadResponseDto>
where
    T: Clone + Default + DeserializeOwned,
{
    let config = device_service
        .config
        .read()
        .map(|c| c.clone())
        .unwrap_or_else(|_| Arc::new(Config::<T>::default()));

    match device_service.read_handler {
        None => Json(ReadResponseDto { data: Type::None }),
        Some(handler) => {
            let data = handler(config).await;
            Json(ReadResponseDto { data })
        }
    }
}

pub(crate) async fn get_config_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
) -> Json<Option<ConfigResponseDto<T>>>
where
    T: DeserializeOwned + Clone + Default,
{
    match read_config_file_with_path(&device_service.config_path) {
        Ok(config) => {
            if let Ok(mut guard) = device_service.config.write() {
                *guard = Arc::new(config.clone());
            }
            Json(Some(ConfigResponseDto::from(config)))
        }
        Err(_) => Json(None),
    }
}

pub(crate) async fn status_device_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
) -> Json<DeviceStatusResponseDto>
where
    T: Clone + Default + DeserializeOwned,
{
    let config = device_service
        .config
        .read()
        .map(|c| c.clone())
        .unwrap_or_else(|_| Arc::new(Config::<T>::default()));

    Json((device_service.status_handler)(config).await)
}

pub(crate) async fn config_update_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
    Json(config): Json<ConfigRequestDto<T>>,
) -> StatusCode
where
    T: Serialize + Clone + Default,
{
    let config = (device_service.config_interceptor_handler)(config, {
        device_service
            .config
            .read()
            .map(|c| c.clone())
            .unwrap_or_else(|_| Arc::new(Config::<T>::default()))
    })
    .await;

    if update_config_file_with_path(&config, &device_service.config_path).is_ok() {
        if let Ok(mut guard) = device_service.config.write() {
            *guard = Arc::new(config);
        }
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub(crate) async fn activate_device<T>(
    State(device_service): State<DeviceBuilder<T>>,
    Json(config): Json<ActivateRequestDto>,
) -> StatusCode
where
    T: Clone + Default + Serialize + DeserializeOwned,
{
    let mut base_config: Config<T> = device_service
        .config
        .read()
        .ok()
        .map(|c| (*c).as_ref().clone())
        .unwrap_or_else(|| {
            read_config_file_with_path(&device_service.config_path).unwrap_or_default()
        });

    base_config.scripting_api = Some(ScriptingApi {
        url: config.url,
        token: config.token,
    });

    if update_config_file_with_path(&base_config, &device_service.config_path).is_ok() {
        if let Ok(mut guard) = device_service.config.write() {
            *guard = Arc::new(base_config);
        }
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
