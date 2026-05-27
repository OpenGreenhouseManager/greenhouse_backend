use std::sync::{Arc, Mutex};

use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use greenhouse_core::smart_device_dto::config::TypeOption;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::config::{Config, write_nvs_config, read_nvs_config};
use crate::error::{Error, Result};
use crate::StatusCode;
use greenhouse_core::smart_device_dto::status::DeviceStatusResponseDto;
use greenhouse_core::smart_device_dto::config::ConfigRequestDto;
use greenhouse_core::smart_device_dto::Type;

const NVS_NAMESPACE: &str = "ghse_cfg";

/// Operating mode of the ESP32 device.
#[derive(Debug, Clone)]
pub enum Mode {
    /// Sensor only — exposes `/read`.
    Input(TypeOption),
    /// Actuator only — exposes `/write`.
    Output(TypeOption),
    /// Both sensor and actuator — exposes `/read` and `/write`.
    InputOutput(TypeOption, TypeOption),
    /// Mode not yet determined.
    Unknown,
}

/// Internal shared state passed to each ESP-IDF HTTP handler closure.
pub struct DeviceState<T: Clone + Default> {
    /// Optional read handler — `Some` for output and hybrid devices.
    pub read_handler: Option<Arc<dyn Fn(Arc<Config<T>>) -> Type + Send + Sync>>,
    /// Optional write handler — `Some` for input and hybrid devices.
    pub write_handler: Option<Arc<dyn Fn(Type, Arc<Config<T>>) -> StatusCode + Send + Sync>>,
    /// Status handler — always present.
    pub status_handler: Arc<dyn Fn(Arc<Config<T>>) -> DeviceStatusResponseDto + Send + Sync>,
    /// Config interceptor called on every `/config` POST.
    pub config_interceptor: Arc<dyn Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T> + Send + Sync>,
    /// Current device configuration protected by a `Mutex`.
    pub config: Mutex<Config<T>>,
    /// NVS partition handle for persisting configuration changes.
    pub nvs: Arc<Mutex<EspNvs<NvsDefault>>>,
    /// Operating mode.
    pub mode: Mode,
}

/// Builder for ESP32 smart-device HTTP servers.
///
/// Mirrors [`greenhouse_core::smart_device_interface::device_builder::DeviceBuilder`]
/// but uses synchronous handler functions and NVS-backed configuration. Call
/// [`DeviceBuilder::run`] at the end of `main` to start the HTTP server.
pub struct DeviceBuilder<T: Clone + Default> {
    pub(crate) state: Arc<DeviceState<T>>,
}

impl<T> DeviceBuilder<T>
where
    T: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Constructs a hybrid device (both sensor and actuator).
    ///
    /// Attempts to load existing configuration from NVS. If none is found,
    /// `default_config` is written to NVS and used as the starting configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the NVS partition cannot be opened, or if a stored
    /// config cannot be deserialised.
    pub fn new_hybrid_device<RH, WH, SH, CIH>(
        nvs_partition: EspDefaultNvsPartition,
        default_config: Config<T>,
        read_handler: RH,
        write_handler: WH,
        status_handler: SH,
        config_interceptor: CIH,
        input_type: TypeOption,
        output_type: TypeOption,
    ) -> Result<Self>
    where
        RH: Fn(Arc<Config<T>>) -> Type + Send + Sync + 'static,
        WH: Fn(Type, Arc<Config<T>>) -> StatusCode + Send + Sync + 'static,
        SH: Fn(Arc<Config<T>>) -> DeviceStatusResponseDto + Send + Sync + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T> + Send + Sync + 'static,
    {
        let (config, nvs) = load_or_init_config(nvs_partition, default_config)?;
        Ok(Self {
            state: Arc::new(DeviceState {
                read_handler: Some(Arc::new(read_handler)),
                write_handler: Some(Arc::new(write_handler)),
                status_handler: Arc::new(status_handler),
                config_interceptor: Arc::new(config_interceptor),
                config: Mutex::new(config),
                nvs,
                mode: Mode::InputOutput(input_type, output_type),
            }),
        })
    }

    /// Constructs an output device (sensor / read-only).
    ///
    /// Exposes `/read`, `/status`, `/config`, and `/activate`.
    ///
    /// # Errors
    ///
    /// Returns an error if the NVS partition cannot be opened.
    pub fn new_output_device<RH, SH, CIH>(
        nvs_partition: EspDefaultNvsPartition,
        default_config: Config<T>,
        read_handler: RH,
        status_handler: SH,
        config_interceptor: CIH,
        output_type: TypeOption,
    ) -> Result<Self>
    where
        RH: Fn(Arc<Config<T>>) -> Type + Send + Sync + 'static,
        SH: Fn(Arc<Config<T>>) -> DeviceStatusResponseDto + Send + Sync + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T> + Send + Sync + 'static,
    {
        let (config, nvs) = load_or_init_config(nvs_partition, default_config)?;
        Ok(Self {
            state: Arc::new(DeviceState {
                read_handler: Some(Arc::new(read_handler)),
                write_handler: None,
                status_handler: Arc::new(status_handler),
                config_interceptor: Arc::new(config_interceptor),
                config: Mutex::new(config),
                nvs,
                mode: Mode::Output(output_type),
            }),
        })
    }

    /// Constructs an input device (actuator / write-only).
    ///
    /// Exposes `/write`, `/status`, `/config`, and `/activate`.
    ///
    /// # Errors
    ///
    /// Returns an error if the NVS partition cannot be opened.
    pub fn new_input_device<WH, SH, CIH>(
        nvs_partition: EspDefaultNvsPartition,
        default_config: Config<T>,
        write_handler: WH,
        status_handler: SH,
        config_interceptor: CIH,
        input_type: TypeOption,
    ) -> Result<Self>
    where
        WH: Fn(Type, Arc<Config<T>>) -> StatusCode + Send + Sync + 'static,
        SH: Fn(Arc<Config<T>>) -> DeviceStatusResponseDto + Send + Sync + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T> + Send + Sync + 'static,
    {
        let (config, nvs) = load_or_init_config(nvs_partition, default_config)?;
        Ok(Self {
            state: Arc::new(DeviceState {
                read_handler: None,
                write_handler: Some(Arc::new(write_handler)),
                status_handler: Arc::new(status_handler),
                config_interceptor: Arc::new(config_interceptor),
                config: Mutex::new(config),
                nvs,
                mode: Mode::Input(input_type),
            }),
        })
    }

    /// Starts the ESP-IDF HTTP server and loops forever.
    ///
    /// Call this at the end of `main`. The server listens on the port from
    /// `Config::port` and routes requests to the appropriate handlers based
    /// on [`Mode`].
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP server cannot be started, or if the
    /// device mode is [`Mode::Unknown`].
    pub fn run(self) -> Result<()> {
        let port = self
            .state
            .config
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .port;
        let server_cfg = esp_idf_svc::http::server::Configuration {
            http_port: port,
            ..Default::default()
        };
        let _server = match &self.state.mode {
            Mode::InputOutput(_, _) => crate::hybrid_device::start_server(self.state.clone(), server_cfg)?,
            Mode::Output(_) => crate::output_device::start_server(self.state.clone(), server_cfg)?,
            Mode::Input(_) => crate::input_device::start_server(self.state.clone(), server_cfg)?,
            Mode::Unknown => return Err(Error::IllFormattedConfig),
        };
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }
}

fn load_or_init_config<T>(
    nvs_partition: EspDefaultNvsPartition,
    default_config: Config<T>,
) -> Result<(Config<T>, Arc<Mutex<EspNvs<NvsDefault>>>)>
where
    T: Clone + Default + Serialize + DeserializeOwned,
{
    let mut nvs = EspNvs::new(nvs_partition, NVS_NAMESPACE, true).map_err(Error::Esp)?;
    let config = match read_nvs_config::<T>(&nvs) {
        Ok(cfg) => cfg,
        Err(Error::MissingConfig) | Err(Error::IllFormattedConfig) => {
            write_nvs_config(&mut nvs, &default_config)?;
            default_config
        }
        Err(e) => return Err(e),
    };
    Ok((config, Arc::new(Mutex::new(nvs))))
}
