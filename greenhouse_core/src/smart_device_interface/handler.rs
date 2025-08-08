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
    smart_device_interface::config::{Config, ScriptingApi, update_config_file},
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
    T: Clone + Default,
{
    let config = device_service.config;
    match device_service.write_handler {
        Some(handler) => handler(payload.data, config).await,
        None => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn read_device_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
) -> Json<ReadResponseDto>
where
    T: Clone + Default,
{
    let config = device_service.config;
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
    State(mut device_service): State<DeviceBuilder<T>>,
) -> Json<Option<ConfigResponseDto<T>>>
where
    T: DeserializeOwned + Clone + Default,
{
    match read_config_file_with_path(&device_service.config_path) {
        Ok(config) => {
            device_service.config = Arc::new(config.clone());
            Json(Some(ConfigResponseDto::from(config)))
        }
        Err(_) => Json(None),
    }
}

pub(crate) async fn status_device_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
) -> Json<DeviceStatusResponseDto>
where
    T: Clone + Default,
{
    let config = device_service.config;

    Json((device_service.status_handler)(config).await)
}

pub(crate) async fn config_update_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
    Json(config): Json<ConfigRequestDto<T>>,
) -> StatusCode
where
    T: Serialize + Clone + Default,
{
    let config = (device_service.config_interceptor_handler)(config, device_service.config).await;

    match update_config_file_with_path(&config, &device_service.config_path) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn activate_device<T>(
    State(device_service): State<DeviceBuilder<T>>,
    Json(config): Json<ActivateRequestDto>,
) -> StatusCode
where
    T: Clone + Default + Serialize,
{
    let config_update = Config {
        scripting_api: Some(ScriptingApi {
            url: config.url,
            token: config.token,
        }),
        mode: device_service.config.mode.clone(),
        port: device_service.config.port,
        datasource_id: device_service.config.datasource_id.clone(),
        input_type: device_service.config.input_type,
        output_type: device_service.config.output_type,
        additional_config: device_service.config.additional_config.clone(),
    };

    match update_config_file(&config_update) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
