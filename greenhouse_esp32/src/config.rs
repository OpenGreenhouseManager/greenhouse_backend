use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::error::{Error, Result};

const NVS_CONFIG_KEY: &str = "cfg";
const NVS_BUF_SIZE: usize = 4096;

/// Runtime configuration for an ESP32 smart device.
///
/// Stored in and loaded from ESP32 Non-Volatile Storage (NVS) via
/// [`write_nvs_config`] and [`read_nvs_config`]. The generic parameter `T`
/// holds device-specific fields beyond the standard ones.
///
/// # Example
///
/// ```no_run
/// # use greenhouse_esp32::config::Config;
/// # #[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
/// # struct MyCfg { threshold: f32 }
/// let config: Config<MyCfg> = Config {
///     port: 80,
///     datasource_id: "uuid-here".to_string(),
///     additional_config: MyCfg { threshold: 30.0 },
///     scripting_api: None,
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config<T: Clone + Default> {
    /// Port on which the ESP-IDF HTTP server listens.
    pub port: u16,
    /// UUID identifying this device's data source in the device service.
    pub datasource_id: String,
    /// Device-specific configuration fields.
    pub additional_config: T,
    /// Scripting service connection, populated via the `/activate` endpoint.
    pub scripting_api: Option<ScriptingApi>,
}

/// Scripting service connection details persisted in NVS configuration.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ScriptingApi {
    /// Base URL of the scripting service.
    pub url: String,
    /// Bearer token for scripting service authentication.
    pub token: String,
}

/// Serialises `config` to JSON and writes it to the NVS partition.
///
/// # Errors
///
/// Returns [`Error::SerializationError`] if JSON serialisation fails, or
/// [`Error::Esp`] if the NVS write fails.
pub fn write_nvs_config<T>(nvs: &mut EspNvs<NvsDefault>, config: &Config<T>) -> Result<()>
where
    T: Serialize + Clone + Default,
{
    let json = serde_json::to_vec(config).map_err(|_| Error::SerializationError)?;
    nvs.set_raw(NVS_CONFIG_KEY, &json).map_err(Error::Esp)?;
    Ok(())
}

/// Reads and deserialises the device configuration from the NVS partition.
///
/// # Errors
///
/// Returns [`Error::MissingConfig`] if no config has been written yet,
/// [`Error::IllFormattedConfig`] if the stored JSON cannot be deserialised,
/// or [`Error::Esp`] on NVS read failure.
pub fn read_nvs_config<T>(nvs: &EspNvs<NvsDefault>) -> Result<Config<T>>
where
    T: DeserializeOwned + Clone + Default,
{
    let mut buf = vec![0u8; NVS_BUF_SIZE];
    match nvs.get_raw(NVS_CONFIG_KEY, &mut buf).map_err(Error::Esp)? {
        Some(data) => serde_json::from_slice(data).map_err(|_| Error::IllFormattedConfig),
        None => Err(Error::MissingConfig),
    }
}
