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

/// Sealed trait alias for futures returned by a read handler.
///
/// Any `Future<Output = Type>` that is `Send + 'static` satisfies this bound
/// automatically via the blanket `impl`. You do not need to name this trait
/// explicitly.
pub trait ReadFuture: Future<Output = Type> + Send + 'static {}
impl<T> ReadFuture for T where T: Future<Output = Type> + Send + 'static {}

/// Sealed trait alias for futures returned by a write handler.
///
/// The output `StatusCode` signals success or failure to the caller.
pub trait WriteFuture: Future<Output = StatusCode> + Send + 'static {}
impl<T> WriteFuture for T where T: Future<Output = StatusCode> + Send + 'static {}

/// Sealed trait alias for futures returned by a status handler.
pub trait StatusFuture: Future<Output = DeviceStatusResponseDto> + Send + 'static {}
impl<T> StatusFuture for T where T: Future<Output = DeviceStatusResponseDto> + Send + 'static {}

/// Sealed trait alias for futures returned by a config interceptor handler.
///
/// The handler receives the incoming [`ConfigRequestDto`] and the current
/// [`Config`], applies any device-specific logic, and returns the new
/// `Config<C>` to be persisted.
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

/// Sealed trait alias for a read handler function.
///
/// A read handler takes a shared reference to the current [`Config`] and
/// returns a [`ReadFuture`] resolving to the current sensor [`Type`] value.
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

/// Sealed trait alias for a write handler function.
///
/// A write handler receives the value to write and a shared [`Config`], then
/// returns a [`WriteFuture`] resolving to a `StatusCode` indicating success or
/// failure.
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

/// Sealed trait alias for a status handler function.
///
/// A status handler takes a shared [`Config`] and returns a
/// [`StatusFuture`] resolving to a [`DeviceStatusResponseDto`].
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

/// Sealed trait alias for a config interceptor handler function.
///
/// The interceptor is called on every `/config` POST. It receives the incoming
/// request DTO and a snapshot of the current config, applies device-specific
/// validation or transformation, and returns the new config to persist.
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

/// Builder for smart device HTTP servers.
///
/// `DeviceBuilder<T>` holds the handler functions and shared configuration
/// needed to drive a smart-device HTTP server. Pass the built value to one of
/// the router init functions ([`init_input_router`](super::input_device::init_input_router),
/// [`init_output_router`](super::output_device::init_output_router), or
/// [`init_hybrid_router`](super::hybrid_device::init_hybrid_router)) to obtain
/// an [`axum::Router`] ready for serving.
///
/// Construct a `DeviceBuilder` using one of the three static constructors:
/// [`new_hybrid_device`](Self::new_hybrid_device),
/// [`new_output_device`](Self::new_output_device), or
/// [`new_input_device`](Self::new_input_device).
#[derive(Clone)]
pub struct DeviceBuilder<T>
where
    T: Clone + Default,
{
    /// Optional read handler — `Some` for output and hybrid devices, `None` for input devices.
    pub read_handler: ReadHandler<T>,
    /// Optional write handler — `Some` for input and hybrid devices, `None` for output devices.
    pub write_handler: WriteHandler<T>,
    /// Status handler — always present.
    pub status_handler: StatusHandler<T>,
    /// Config interceptor called on every `/config` POST.
    pub config_interceptor_handler: ConfigInterceptorHandler<T>,
    /// Shared, live device configuration protected by an `RwLock`.
    pub config: Arc<RwLock<Arc<Config<T>>>>,
    /// Path to the JSON configuration file used for persistence.
    pub config_path: String,
    /// The operating mode reported by the `/config` endpoint.
    pub mode: Mode,
}

impl<T> DeviceBuilder<T>
where
    T: Clone + Default + DeserializeOwned + Serialize,
{
    /// Constructs a hybrid device (both sensor and actuator) using the default config path.
    ///
    /// Reads the configuration from `./config/config.json`.
    ///
    /// # Type Parameters
    ///
    /// - `RH` / `RF` — read handler function and its future type
    /// - `WH` / `WF` — write handler function and its future type
    /// - `SH` / `SF` — status handler function and its future type
    /// - `CIH` / `CIF` — config interceptor function and its future type
    ///
    /// # Errors
    ///
    /// Returns [`super::Error::MissingConfig`] if `./config/config.json` does not exist,
    /// or [`super::Error::IllFormattedConfig`] if the file cannot be deserialised.
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

    /// Constructs a hybrid device using a custom configuration file path.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error::MissingConfig`] if `config_path` does not exist, or
    /// [`super::Error::IllFormattedConfig`] if the file cannot be deserialised.
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

    /// Constructs an output device (sensor / read-only) using the default config path.
    ///
    /// An output device exposes `/read`, `/status`, `/config`, and `/activate`.
    /// If the config file does not exist, a default `Config<T>` is written to
    /// `./config/config.json` before starting.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error::IllFormattedConfig`] if an existing config file cannot be
    /// parsed, or [`super::Error::MissingConfig`] if the default config cannot be written.
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

    /// Constructs an output device using a custom configuration file path.
    ///
    /// If the file at `config_path` does not exist, a default `Config<T>` is
    /// created and persisted there.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error::IllFormattedConfig`] if an existing config file is
    /// malformed, or [`super::Error::MissingConfig`] if the default config cannot
    /// be written to `config_path`.
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

    /// Constructs an input device (actuator / write-only) using the default config path.
    ///
    /// An input device exposes `/write`, `/status`, `/config`, and `/activate`.
    /// If the config file does not exist, a default `Config<T>` is written before
    /// starting.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error::IllFormattedConfig`] if an existing config file is
    /// malformed, or [`super::Error::MissingConfig`] if the default config cannot be written.
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

    /// Constructs an input device using a custom configuration file path.
    ///
    /// If the file at `config_path` does not exist, a default `Config<T>` is
    /// created and persisted there.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error::IllFormattedConfig`] if an existing config file is
    /// malformed, or [`super::Error::MissingConfig`] if the default config cannot
    /// be written to `config_path`.
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
