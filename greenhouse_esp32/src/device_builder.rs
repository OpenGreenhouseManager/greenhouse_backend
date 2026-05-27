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

#[derive(Debug, Clone)]
pub enum Mode {
    Input(TypeOption),
    Output(TypeOption),
    InputOutput(TypeOption, TypeOption),
    Unknown,
}

pub struct DeviceState<T: Clone + Default> {
    pub read_handler: Option<Arc<dyn Fn(Arc<Config<T>>) -> Type + Send + Sync>>,
    pub write_handler: Option<Arc<dyn Fn(Type, Arc<Config<T>>) -> StatusCode + Send + Sync>>,
    pub status_handler: Arc<dyn Fn(Arc<Config<T>>) -> DeviceStatusResponseDto + Send + Sync>,
    pub config_interceptor: Arc<dyn Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> Config<T> + Send + Sync>,
    pub config: Mutex<Config<T>>,
    pub nvs: Arc<Mutex<EspNvs<NvsDefault>>>,
    pub mode: Mode,
}

pub struct DeviceBuilder<T: Clone + Default> {
    pub(crate) state: Arc<DeviceState<T>>,
}

impl<T> DeviceBuilder<T>
where
    T: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
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

    /// Starts the HTTP server and loops forever. Call this at the end of `main`.
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
