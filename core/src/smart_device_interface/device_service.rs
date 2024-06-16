use crate::smart_device_dto::{config::ConfigRequestDto, status::DeviceStatusResponseDto};
use axum::http::StatusCode;
use serde::de::DeserializeOwned;
use std::sync::Arc;

use super::config::{read_config_file, Config};

#[derive(Clone)]
pub struct DeviceService<T>
where
    T: Clone,
{
    pub read_handler: Option<Arc<dyn (Fn() -> String) + Send + Sync>>,
    pub write_handler: Option<Arc<dyn (Fn(String) -> StatusCode) + Send + Sync>>,
    pub status_handler: Arc<dyn (Fn() -> DeviceStatusResponseDto) + Send + Sync>,
    pub config_interceptor_handler: Arc<dyn (Fn(ConfigRequestDto<T>) -> Config<T>) + Send + Sync>,
    pub config: Arc<Config<T>>,
}

impl<T> DeviceService<T>
where
    T: DeserializeOwned + Clone,
{
    pub fn new_hybrid_device(
        read_handler: impl (Fn() -> String) + Send + Sync + 'static,
        write_handler: impl (Fn(String) -> StatusCode) + Send + Sync + 'static,
        status_handler: impl (Fn() -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>) -> Config<T>) + Send + Sync + 'static,
    ) -> Self {
        let config = read_config_file().unwrap();
        DeviceService {
            read_handler: Some(Arc::new(read_handler)),
            write_handler: Some(Arc::new(write_handler)),
            status_handler: Arc::new(status_handler),
            config_interceptor_handler: Arc::new(config_interceptor_handler),
            config: Arc::new(config),
        }
    }

    pub fn new_output_device(
        read_handler: impl (Fn() -> String) + Send + Sync + 'static,
        status_handler: impl (Fn() -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>) -> Config<T>) + Send + Sync + 'static,
    ) -> Self {
        let config = read_config_file().unwrap();
        DeviceService {
            read_handler: Some(Arc::new(read_handler)),
            write_handler: None,
            status_handler: Arc::new(status_handler),
            config_interceptor_handler: Arc::new(config_interceptor_handler),
            config: Arc::new(config),
        }
    }

    pub fn new_input_device(
        write_handler: impl (Fn(String) -> StatusCode) + Send + Sync + 'static,
        status_handler: impl (Fn() -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>) -> Config<T>) + Send + Sync + 'static,
    ) -> Self {
        let config = read_config_file().unwrap();
        DeviceService {
            read_handler: None,
            write_handler: Some(Arc::new(write_handler)),
            status_handler: Arc::new(status_handler),
            config_interceptor_handler: Arc::new(config_interceptor_handler),
            config: Arc::new(config),
        }
    }
}
