use crate::smart_device_dto::{config::ConfigRequestDto, status::DeviceStatusResponseDto};
use axum::http::StatusCode;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

use super::config::{read_config_file, update_config_file, Config};

type ReadHandler<T> = Option<Arc<dyn (Fn(Arc<Config<T>>) -> String) + Send + Sync>>;
type WriteHandler<T> = Option<Arc<dyn (Fn(String, Arc<Config<T>>) -> StatusCode) + Send + Sync>>;
type StatusHandler<T> = Arc<dyn (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync>;
type ConfigInterceptorHandler<T> = Arc<dyn (Fn(ConfigRequestDto<T>) -> Config<T>) + Send + Sync>;

#[derive(Clone)]
pub struct DeviceService<T>
where
    T: Clone + Default,
{
    pub read_handler: ReadHandler<T>,
    pub write_handler: WriteHandler<T>,
    pub status_handler: StatusHandler<T>,
    pub config_interceptor_handler: ConfigInterceptorHandler<T>,
    pub config: Arc<Config<T>>,
}

impl<T> DeviceService<T>
where
    T: Clone + Default + DeserializeOwned + Serialize,
{
    pub fn new_hybrid_device(
        read_handler: impl (Fn(Arc<Config<T>>) -> String) + Send + Sync + 'static,
        write_handler: impl (Fn(String, Arc<Config<T>>) -> StatusCode) + Send + Sync + 'static,
        status_handler: impl (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>) -> Config<T>) + Send + Sync + 'static,
    ) -> Self {
        let config = match read_config_file() {
            Ok(config) => config,
            Err(_) => {
                let default_config = Config::default();
                update_config_file(&default_config).unwrap();
                default_config
            }
        };
        DeviceService {
            read_handler: Some(Arc::new(read_handler)),
            write_handler: Some(Arc::new(write_handler)),
            status_handler: Arc::new(status_handler),
            config_interceptor_handler: Arc::new(config_interceptor_handler),
            config: Arc::new(config),
        }
    }

    pub fn new_output_device(
        read_handler: impl (Fn(Arc<Config<T>>) -> String) + Send + Sync + 'static,
        status_handler: impl (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>) -> Config<T>) + Send + Sync + 'static,
    ) -> Self {
        let config = match read_config_file() {
            Ok(config) => config,
            Err(_) => {
                let default_config = Config::default();
                update_config_file(&default_config).unwrap();
                default_config
            }
        };
        DeviceService {
            read_handler: Some(Arc::new(read_handler)),
            write_handler: None,
            status_handler: Arc::new(status_handler),
            config_interceptor_handler: Arc::new(config_interceptor_handler),
            config: Arc::new(config),
        }
    }

    pub fn new_input_device(
        write_handler: impl (Fn(String, Arc<Config<T>>) -> StatusCode) + Send + Sync + 'static,
        status_handler: impl (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>) -> Config<T>) + Send + Sync + 'static,
    ) -> Self {
        let config = match read_config_file() {
            Ok(config) => config,
            Err(_) => {
                let default_config = Config::default();
                update_config_file(&default_config).unwrap();
                default_config
            }
        };
        DeviceService {
            read_handler: None,
            write_handler: Some(Arc::new(write_handler)),
            status_handler: Arc::new(status_handler),
            config_interceptor_handler: Arc::new(config_interceptor_handler),
            config: Arc::new(config),
        }
    }
}
