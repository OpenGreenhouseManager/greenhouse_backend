use super::Result;
use super::config::{
    Config, DEFAULT_CONFIG_FILE_NAME, read_config_file_with_path, update_config_file_with_path,
};
use crate::smart_device_dto::{config::ConfigRequestDto, status::DeviceStatusResponseDto};
use axum::http::StatusCode;
use futures::future::BoxFuture;
use serde::{Serialize, de::DeserializeOwned};
use std::future::Future;
use std::sync::Arc;

type ReadHandler<T> =
    Option<Arc<dyn Fn(Arc<Config<T>>) -> BoxFuture<'static, String> + Send + Sync>>;
type WriteHandler<T> =
    Option<Arc<dyn Fn(String, Arc<Config<T>>) -> BoxFuture<'static, StatusCode> + Send + Sync>>;
type StatusHandler<T> =
    Arc<dyn Fn(Arc<Config<T>>) -> BoxFuture<'static, DeviceStatusResponseDto> + Send + Sync>;
type ConfigInterceptorHandler<T> =
    Arc<dyn Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> BoxFuture<'static, Config<T>> + Send + Sync>;

#[derive(Clone)]
pub struct DeviceBuilder<T>
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

impl<T> DeviceBuilder<T>
where
    T: Clone + Default + DeserializeOwned + Serialize,
{
    pub fn new_hybrid_device<RH, RF, WH, WF, SH, SF, CIH, CIF>(
        read_handler: RH,
        write_handler: WH,
        status_handler: SH,
        config_interceptor_handler: CIH,
    ) -> Result<Self>
    where
        RH: Fn(Arc<Config<T>>) -> RF + Send + Sync + 'static,
        RF: Future<Output = String> + Send + 'static,
        WH: Fn(String, Arc<Config<T>>) -> WF + Send + Sync + 'static,
        WF: Future<Output = StatusCode> + Send + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + Send + Sync + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + Send + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + Send + Sync + 'static,
        CIF: Future<Output = Config<T>> + Send + 'static,
    {
        Self::new_hybrid_device_with_config_path(
            read_handler,
            write_handler,
            status_handler,
            config_interceptor_handler,
            DEFAULT_CONFIG_FILE_NAME,
        )
    }

    pub fn new_hybrid_device_with_config_path<RH, RF, WH, WF, SH, SF, CIH, CIF>(
        read_handler: RH,
        write_handler: WH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        config_path: &str,
    ) -> Result<Self>
    where
        RH: Fn(Arc<Config<T>>) -> RF + Send + Sync + 'static,
        RF: Future<Output = String> + Send + 'static,
        WH: Fn(String, Arc<Config<T>>) -> WF + Send + Sync + 'static,
        WF: Future<Output = StatusCode> + Send + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + Send + Sync + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + Send + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + Send + Sync + 'static,
        CIF: Future<Output = Config<T>> + Send + 'static,
    {
        let config = read_config_file_with_path(config_path)?;

        Ok(DeviceBuilder {
            read_handler: Some(Arc::new(move |cfg: Arc<Config<T>>| {
                let fut = read_handler(cfg);
                Box::pin(fut)
            })),
            write_handler: Some(Arc::new(move |data: String, cfg: Arc<Config<T>>| {
                let fut = write_handler(data, cfg);
                Box::pin(fut)
            })),
            status_handler: Arc::new(move |cfg: Arc<Config<T>>| {
                let fut = status_handler(cfg);
                Box::pin(fut)
            }),
            config_interceptor_handler: Arc::new(
                move |req: ConfigRequestDto<T>, cfg: Arc<Config<T>>| {
                    let fut = config_interceptor_handler(req, cfg);
                    Box::pin(fut)
                },
            ),
            config: Arc::new(config),
            config_path: config_path.to_string(),
        })
    }

    pub fn new_output_device<RH, RF, SH, SF, CIH, CIF>(
        read_handler: RH,
        status_handler: SH,
        config_interceptor_handler: CIH,
    ) -> Result<Self>
    where
        RH: Fn(Arc<Config<T>>) -> RF + Send + Sync + 'static,
        RF: Future<Output = String> + Send + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + Send + Sync + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + Send + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + Send + Sync + 'static,
        CIF: Future<Output = Config<T>> + Send + 'static,
    {
        Self::new_output_device_with_config_path(
            read_handler,
            status_handler,
            config_interceptor_handler,
            DEFAULT_CONFIG_FILE_NAME,
        )
    }

    pub fn new_output_device_with_config_path<RH, RF, SH, SF, CIH, CIF>(
        read_handler: RH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        config_path: &str,
    ) -> Result<Self>
    where
        RH: Fn(Arc<Config<T>>) -> RF + Send + Sync + 'static,
        RF: Future<Output = String> + Send + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + Send + Sync + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + Send + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + Send + Sync + 'static,
        CIF: Future<Output = Config<T>> + Send + 'static,
    {
        let config = match read_config_file_with_path(config_path) {
            Ok(config) => config,
            Err(_) => {
                let default_config = Config::default();
                update_config_file_with_path(&default_config, config_path)?;
                default_config
            }
        };
        Ok(DeviceBuilder {
            read_handler: Some(Arc::new(move |cfg: Arc<Config<T>>| {
                let fut = read_handler(cfg);
                Box::pin(fut)
            })),
            write_handler: None,
            status_handler: Arc::new(move |cfg: Arc<Config<T>>| {
                let fut = status_handler(cfg);
                Box::pin(fut)
            }),
            config_interceptor_handler: Arc::new(
                move |req: ConfigRequestDto<T>, cfg: Arc<Config<T>>| {
                    let fut = config_interceptor_handler(req, cfg);
                    Box::pin(fut)
                },
            ),
            config: Arc::new(config),
            config_path: config_path.to_string(),
        })
    }

    pub fn new_input_device<WH, WF, SH, SF, CIH, CIF>(
        write_handler: WH,
        status_handler: SH,
        config_interceptor_handler: CIH,
    ) -> Result<Self>
    where
        WH: Fn(String, Arc<Config<T>>) -> WF + Send + Sync + 'static,
        WF: Future<Output = StatusCode> + Send + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + Send + Sync + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + Send + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + Send + Sync + 'static,
        CIF: Future<Output = Config<T>> + Send + 'static,
    {
        Self::new_input_device_with_config_path(
            write_handler,
            status_handler,
            config_interceptor_handler,
            DEFAULT_CONFIG_FILE_NAME,
        )
    }

    pub fn new_input_device_with_config_path<WH, WF, SH, SF, CIH, CIF>(
        write_handler: WH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        config_path: &str,
    ) -> Result<Self>
    where
        WH: Fn(String, Arc<Config<T>>) -> WF + Send + Sync + 'static,
        WF: Future<Output = StatusCode> + Send + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + Send + Sync + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + Send + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + Send + Sync + 'static,
        CIF: Future<Output = Config<T>> + Send + 'static,
    {
        let config = match read_config_file_with_path(config_path) {
            Ok(config) => config,
            Err(_) => {
                let default_config = Config::default();
                update_config_file_with_path(&default_config, config_path)?;
                default_config
            }
        };
        Ok(DeviceBuilder {
            read_handler: None,
            write_handler: Some(Arc::new(move |data: String, cfg: Arc<Config<T>>| {
                let fut = write_handler(data, cfg);
                Box::pin(fut)
            })),
            status_handler: Arc::new(move |cfg: Arc<Config<T>>| {
                let fut = status_handler(cfg);
                Box::pin(fut)
            }),
            config_interceptor_handler: Arc::new(
                move |req: ConfigRequestDto<T>, cfg: Arc<Config<T>>| {
                    let fut = config_interceptor_handler(req, cfg);
                    Box::pin(fut)
                },
            ),
            config: Arc::new(config),
            config_path: config_path.to_string(),
        })
    }
}
