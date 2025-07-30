use super::Result;
use super::config::{Config, read_config_file_with_path, update_config_file_with_path, DEFAULT_CONFIG_FILE_NAME};
use crate::smart_device_dto::{config::ConfigRequestDto, status::DeviceStatusResponseDto};
use axum::http::StatusCode;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

type ReadHandler<T> = Option<Arc<dyn (Fn(Arc<Config<T>>) -> String) + Send + Sync>>;
type WriteHandler<T> = Option<Arc<dyn (Fn(String, Arc<Config<T>>) -> StatusCode) + Send + Sync>>;
type StatusHandler<T> = Arc<dyn (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync>;
type ConfigInterceptorHandler<T> =
    Arc<dyn (Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T>) + Send + Sync>;

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
    pub config_path: String,
}

impl<T> DeviceService<T>
where
    T: Clone + Default + DeserializeOwned + Serialize,
{
    pub fn new_hybrid_device(
        read_handler: impl (Fn(Arc<Config<T>>) -> String) + Send + Sync + 'static,
        write_handler: impl (Fn(String, Arc<Config<T>>) -> StatusCode) + Send + Sync + 'static,
        status_handler: impl (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T>)
        + Send
        + Sync
        + 'static,
    ) -> Result<Self> {
        Self::new_hybrid_device_with_config_path(
            read_handler,
            write_handler,
            status_handler,
            config_interceptor_handler,
            DEFAULT_CONFIG_FILE_NAME,
        )
    }

    pub fn new_hybrid_device_with_config_path(
        read_handler: impl (Fn(Arc<Config<T>>) -> String) + Send + Sync + 'static,
        write_handler: impl (Fn(String, Arc<Config<T>>) -> StatusCode) + Send + Sync + 'static,
        status_handler: impl (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T>)
        + Send
        + Sync
        + 'static,
        config_path: &str,
    ) -> Result<Self> {
        let config = read_config_file_with_path(config_path)?;

        Ok(DeviceService {
            read_handler: Some(Arc::new(read_handler)),
            write_handler: Some(Arc::new(write_handler)),
            status_handler: Arc::new(status_handler),
            config_interceptor_handler: Arc::new(config_interceptor_handler),
            config: Arc::new(config),
            config_path: config_path.to_string(),
        })
    }

    pub fn new_output_device(
        read_handler: impl (Fn(Arc<Config<T>>) -> String) + Send + Sync + 'static,
        status_handler: impl (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T>)
        + Send
        + Sync
        + 'static,
    ) -> Result<Self> {
        Self::new_output_device_with_config_path(
            read_handler,
            status_handler,
            config_interceptor_handler,
            DEFAULT_CONFIG_FILE_NAME,
        )
    }

    pub fn new_output_device_with_config_path(
        read_handler: impl (Fn(Arc<Config<T>>) -> String) + Send + Sync + 'static,
        status_handler: impl (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T>)
        + Send
        + Sync
        + 'static,
        config_path: &str,
    ) -> Result<Self> {
        let config = match read_config_file_with_path(config_path) {
            Ok(config) => config,
            Err(_) => {
                let default_config = Config::default();
                update_config_file_with_path(&default_config, config_path)?;
                default_config
            }
        };
        Ok(DeviceService {
            read_handler: Some(Arc::new(read_handler)),
            write_handler: None,
            status_handler: Arc::new(status_handler),
            config_interceptor_handler: Arc::new(config_interceptor_handler),
            config: Arc::new(config),
            config_path: config_path.to_string(),
        })
    }

    pub fn new_input_device(
        write_handler: impl (Fn(String, Arc<Config<T>>) -> StatusCode) + Send + Sync + 'static,
        status_handler: impl (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T>)
        + Send
        + Sync
        + 'static,
    ) -> Result<Self> {
        Self::new_input_device_with_config_path(
            write_handler,
            status_handler,
            config_interceptor_handler,
            DEFAULT_CONFIG_FILE_NAME,
        )
    }

    pub fn new_input_device_with_config_path(
        write_handler: impl (Fn(String, Arc<Config<T>>) -> StatusCode) + Send + Sync + 'static,
        status_handler: impl (Fn(Arc<Config<T>>) -> DeviceStatusResponseDto) + Send + Sync + 'static,
        config_interceptor_handler: impl (Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T>)
        + Send
        + Sync
        + 'static,
        config_path: &str,
    ) -> Result<Self> {
        let config = match read_config_file_with_path(config_path) {
            Ok(config) => config,
            Err(_) => {
                let default_config = Config::default();
                update_config_file_with_path(&default_config, config_path)?;
                default_config
            }
        };
        Ok(DeviceService {
            read_handler: None,
            write_handler: Some(Arc::new(write_handler)),
            status_handler: Arc::new(status_handler),
            config_interceptor_handler: Arc::new(config_interceptor_handler),
            config: Arc::new(config),
            config_path: config_path.to_string(),
        })
    }
}
