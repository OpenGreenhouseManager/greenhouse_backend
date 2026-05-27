use crate::smart_device_dto::config::TypeOption;

use super::{Error, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// Default path used by [`read_config_file`] and [`update_config_file`].
pub(crate) const DEFAULT_CONFIG_FILE_NAME: &str = "./config/config.json";

/// Reads the device configuration from the default path (`./config/config.json`).
///
/// Returns [`super::Error::MissingConfig`] if the file does not exist and
/// [`super::Error::IllFormattedConfig`] if the file cannot be deserialised.
///
/// ```no_run
/// # use greenhouse_core::smart_device_interface::config::{read_config_file, Config};
/// # #[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
/// # struct MyConfig {}
/// let config: Config<MyConfig> = read_config_file().unwrap();
/// println!("Listening on port {}", config.port);
/// ```
pub fn read_config_file<T>() -> Result<Config<T>>
where
    T: DeserializeOwned + Clone + Default,
{
    read_config_file_with_path(DEFAULT_CONFIG_FILE_NAME)
}

/// Writes the device configuration to the default path (`./config/config.json`).
///
/// Overwrites the file if it already exists.
///
/// ```no_run
/// # use greenhouse_core::smart_device_interface::config::{update_config_file, Config};
/// # #[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
/// # struct MyConfig {}
/// let config: Config<MyConfig> = Config::default();
/// update_config_file(&config).unwrap();
/// ```
pub fn update_config_file<T>(config: &Config<T>) -> Result<()>
where
    T: Serialize + Clone + Default,
{
    update_config_file_with_path(config, DEFAULT_CONFIG_FILE_NAME)
}

/// Writes the device configuration to a specified file path.
///
/// # Errors
///
/// Returns [`super::Error::IllFormattedConfig`] if serialisation fails, or
/// [`super::Error::MissingConfig`] if the file cannot be written (e.g. directory does
/// not exist).
pub fn update_config_file_with_path<T>(config: &Config<T>, config_path: &str) -> Result<()>
where
    T: Serialize + Clone + Default,
{
    let json_string = serde_json::to_string(&config).map_err(|_| Error::IllFormattedConfig)?;
    std::fs::write(config_path, json_string).map_err(|_| Error::MissingConfig)
}

/// Reads the device configuration from a specified file path.
///
/// # Errors
///
/// Returns [`super::Error::MissingConfig`] if the file does not exist, or
/// [`super::Error::IllFormattedConfig`] if the JSON cannot be deserialised into
/// `Config<T>`.
pub fn read_config_file_with_path<T>(config_path: &str) -> Result<Config<T>>
where
    T: DeserializeOwned + Clone + Default,
{
    let data = std::fs::read_to_string(config_path).map_err(|_| Error::MissingConfig)?;
    serde_json::from_str(&data).map_err(|_| Error::IllFormattedConfig)
}

/// The operating mode stored in a device's configuration file.
///
/// Controls which endpoints [`DeviceBuilder`](super::device_builder::DeviceBuilder)
/// wires up at startup.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Mode {
    /// Sensor (produces readings) — exposes `/read`.
    Input(TypeOption),
    /// Actuator (accepts commands) — exposes `/write`.
    Output(TypeOption),
    /// Both sensor and actuator — exposes `/read` and `/write`.
    InputOutput(TypeOption, TypeOption),
    /// Mode has not been determined yet.
    #[default]
    Unknown,
}

/// Type option variant stored in the configuration file.
///
/// Mirrors [`smart_device_dto::config::TypeOption`](crate::smart_device_dto::config::TypeOption)
/// for serialisation purposes.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub enum TypeOptionDto {
    /// Numeric (`f64`) value.
    Number,
    /// Boolean value.
    Boolean,
    /// Key-value object.
    Object,
    /// Physical measurement with unit.
    Measurement,
    /// Data stream.
    Stream,
    /// Type not yet known.
    #[default]
    Unknown,
}

/// Runtime configuration for a smart device, loaded from a JSON file.
///
/// The generic parameter `T` holds device-specific fields beyond the common ones.
/// Create a `Config<T>` by implementing `Default` on `T` and calling
/// [`read_config_file`].
///
/// # Example
///
/// ```
/// # use greenhouse_core::smart_device_interface::config::Config;
/// # #[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
/// # struct MyConfig { threshold: f32 }
/// let config: Config<MyConfig> = Config {
///     port: 8080,
///     datasource_id: "uuid-here".to_string(),
///     additional_config: MyConfig { threshold: 25.0 },
///     scripting_api: None,
/// };
/// assert_eq!(config.port, 8080);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config<T>
where
    T: Clone + Default,
{
    /// Port on which the device's HTTP server listens.
    pub port: u16,
    /// UUID that identifies this device's data source in the device service.
    pub datasource_id: String,
    /// Device-specific configuration fields.
    pub additional_config: T,
    /// Scripting service connection, populated by the `/activate` endpoint.
    pub scripting_api: Option<ScriptingApi>,
}

/// Scripting service connection details persisted to device configuration.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ScriptingApi {
    /// Base URL of the scripting service.
    pub url: String,
    /// Bearer token for scripting service authentication.
    pub token: String,
}
