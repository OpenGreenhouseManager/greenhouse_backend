use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    smart_device_dto::{
        self,
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
    // Use latest in-memory config
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
    // Use latest in-memory config
    let config = device_service
        .config
        .read()
        .map(|c| c.clone())
        .unwrap_or_else(|_| Arc::new(Config::<T>::default()));

    let output_type = match config.output_type {
        Some(output_type) => output_type.into(),
        None => smart_device_dto::Type::Unknown,
    };
    match device_service.read_handler {
        None => Json(Default::default()),
        Some(handler) => {
            let data = handler(config).await;
            Json(ReadResponseDto { data, output_type })
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
            // Update in-memory config
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
    // Use latest in-memory config
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

    // Persist to disk
    if update_config_file_with_path(&config, &device_service.config_path).is_ok() {
        // Update in-memory config
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
    // Start from in-memory config, falling back to disk if poisoned/empty
    let mut base_config: Config<T> = device_service
        .config
        .read()
        .ok()
        .map(|c| (*c).as_ref().clone())
        .unwrap_or_else(|| {
            read_config_file_with_path(&device_service.config_path).unwrap_or_default()
        });

    // Update only the scripting_api in the config
    base_config.scripting_api = Some(ScriptingApi {
        url: config.url,
        token: config.token,
    });

    // Persist and refresh in-memory
    if update_config_file_with_path(&base_config, &device_service.config_path).is_ok() {
        if let Ok(mut guard) = device_service.config.write() {
            *guard = Arc::new(base_config);
        }
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
