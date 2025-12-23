use super::Result;
use super::config::{
    Config, DEFAULT_CONFIG_FILE_NAME, read_config_file_with_path, update_config_file_with_path,
};
use crate::smart_device_dto::Type;
use crate::smart_device_dto::config::TypeOption;
use crate::smart_device_dto::{config::ConfigRequestDto, status::DeviceStatusResponseDto};
use crate::smart_device_interface::config::Mode;
use axum::http::StatusCode;
use futures::future::BoxFuture;
use serde::{Serialize, de::DeserializeOwned};
use std::future::Future;
use std::sync::{Arc, RwLock};

// Trait aliases for repetitive future bounds
pub trait ReadFuture: Future<Output = Type> + Send + 'static {}
impl<T> ReadFuture for T where T: Future<Output = Type> + Send + 'static {}

pub trait WriteFuture: Future<Output = StatusCode> + Send + 'static {}
impl<T> WriteFuture for T where T: Future<Output = StatusCode> + Send + 'static {}

pub trait StatusFuture: Future<Output = DeviceStatusResponseDto> + Send + 'static {}
impl<T> StatusFuture for T where T: Future<Output = DeviceStatusResponseDto> + Send + 'static {}

pub trait ConfigFuture<C>: Future<Output = Config<C>> + Send + 'static
where
    C: Clone + Default,
{
}
impl<T, C> ConfigFuture<C> for T
where
    T: Future<Output = Config<C>> + Send + 'static,
    C: Clone + Default,
{
}

// Trait aliases for repetitive function signature patterns
pub trait ReadHandlerFn<T, RF>: Fn(Arc<Config<T>>) -> RF + Send + Sync + 'static
where
    T: Clone + Default,
{
}
impl<F, T, RF> ReadHandlerFn<T, RF> for F
where
    F: Fn(Arc<Config<T>>) -> RF + Send + Sync + 'static,
    T: Clone + Default,
{
}

pub trait WriteHandlerFn<T, WF>: Fn(Type, Arc<Config<T>>) -> WF + Send + Sync + 'static
where
    T: Clone + Default,
{
}
impl<F, T, WF> WriteHandlerFn<T, WF> for F
where
    F: Fn(Type, Arc<Config<T>>) -> WF + Send + Sync + 'static,
    T: Clone + Default,
{
}

pub trait StatusHandlerFn<T, SF>: Fn(Arc<Config<T>>) -> SF + Send + Sync + 'static
where
    T: Clone + Default,
{
}
impl<F, T, SF> StatusHandlerFn<T, SF> for F
where
    F: Fn(Arc<Config<T>>) -> SF + Send + Sync + 'static,
    T: Clone + Default,
{
}

pub trait ConfigHandlerFn<T, CIF>:
    Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + Send + Sync + 'static
where
    T: Clone + Default,
{
}
impl<F, T, CIF> ConfigHandlerFn<T, CIF> for F
where
    F: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + Send + Sync + 'static,
    T: Clone + Default,
{
}

type ReadHandler<T> = Option<Arc<dyn Fn(Arc<Config<T>>) -> BoxFuture<'static, Type> + Send + Sync>>;
type WriteHandler<T> =
    Option<Arc<dyn Fn(Type, Arc<Config<T>>) -> BoxFuture<'static, StatusCode> + Send + Sync>>;
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
    pub config: Arc<RwLock<Arc<Config<T>>>>,
    pub config_path: String,
    pub mode: Mode,
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
        input_type: TypeOption,
        output_type: TypeOption,
    ) -> Result<Self>
    where
        RH: ReadHandlerFn<T, RF>,
        RF: ReadFuture,
        WH: WriteHandlerFn<T, WF>,
        WF: WriteFuture,
        SH: StatusHandlerFn<T, SF>,
        SF: StatusFuture,
        CIH: ConfigHandlerFn<T, CIF>,
        CIF: ConfigFuture<T>,
    {
        Self::new_hybrid_device_with_config_path(
            read_handler,
            write_handler,
            status_handler,
            config_interceptor_handler,
            DEFAULT_CONFIG_FILE_NAME,
            input_type,
            output_type,
        )
    }

    pub fn new_hybrid_device_with_config_path<RH, RF, WH, WF, SH, SF, CIH, CIF>(
        read_handler: RH,
        write_handler: WH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        config_path: &str,
        input_type: TypeOption,
        output_type: TypeOption,
    ) -> Result<Self>
    where
        RH: ReadHandlerFn<T, RF>,
        RF: ReadFuture,
        WH: WriteHandlerFn<T, WF>,
        WF: WriteFuture,
        SH: StatusHandlerFn<T, SF>,
        SF: StatusFuture,
        CIH: ConfigHandlerFn<T, CIF>,
        CIF: ConfigFuture<T>,
    {
        let config = read_config_file_with_path(config_path)?;

        Ok(DeviceBuilder {
            read_handler: Some(Arc::new(move |cfg: Arc<Config<T>>| {
                let fut = read_handler(cfg);
                Box::pin(fut)
            })),
            write_handler: Some(Arc::new(move |data: Type, cfg: Arc<Config<T>>| {
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
            config: Arc::new(RwLock::new(Arc::new(config))),
            config_path: config_path.to_string(),
            mode: Mode::InputOutput(input_type, output_type),
        })
    }

    pub fn new_output_device<RH, RF, SH, SF, CIH, CIF>(
        read_handler: RH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        output_type: TypeOption,
    ) -> Result<Self>
    where
        RH: ReadHandlerFn<T, RF>,
        RF: ReadFuture,
        SH: StatusHandlerFn<T, SF>,
        SF: StatusFuture,
        CIH: ConfigHandlerFn<T, CIF>,
        CIF: ConfigFuture<T>,
    {
        Self::new_output_device_with_config_path(
            read_handler,
            status_handler,
            config_interceptor_handler,
            DEFAULT_CONFIG_FILE_NAME,
            output_type,
        )
    }

    pub fn new_output_device_with_config_path<RH, RF, SH, SF, CIH, CIF>(
        read_handler: RH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        config_path: &str,
        output_type: TypeOption,
    ) -> Result<Self>
    where
        RH: ReadHandlerFn<T, RF>,
        RF: ReadFuture,
        SH: StatusHandlerFn<T, SF>,
        SF: StatusFuture,
        CIH: ConfigHandlerFn<T, CIF>,
        CIF: ConfigFuture<T>,
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
            config: Arc::new(RwLock::new(Arc::new(config))),
            config_path: config_path.to_string(),
            mode: Mode::Output(output_type),
        })
    }

    pub fn new_input_device<WH, WF, SH, SF, CIH, CIF>(
        write_handler: WH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        input_type: TypeOption,
    ) -> Result<Self>
    where
        WH: WriteHandlerFn<T, WF>,
        WF: WriteFuture,
        SH: StatusHandlerFn<T, SF>,
        SF: StatusFuture,
        CIH: ConfigHandlerFn<T, CIF>,
        CIF: ConfigFuture<T>,
    {
        Self::new_input_device_with_config_path(
            write_handler,
            status_handler,
            config_interceptor_handler,
            DEFAULT_CONFIG_FILE_NAME,
            input_type,
        )
    }

    pub fn new_input_device_with_config_path<WH, WF, SH, SF, CIH, CIF>(
        write_handler: WH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        config_path: &str,
        input_type: TypeOption,
    ) -> Result<Self>
    where
        WH: WriteHandlerFn<T, WF>,
        WF: WriteFuture,
        SH: StatusHandlerFn<T, SF>,
        SF: StatusFuture,
        CIH: ConfigHandlerFn<T, CIF>,
        CIF: ConfigFuture<T>,
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
            write_handler: Some(Arc::new(move |data: Type, cfg: Arc<Config<T>>| {
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
            config: Arc::new(RwLock::new(Arc::new(config))),
            config_path: config_path.to_string(),
            mode: Mode::Input(input_type),
        })
    }
}
