use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::error::{Error, Result};

const NVS_CONFIG_KEY: &str = "cfg";
const NVS_BUF_SIZE: usize = 4096;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config<T: Clone + Default> {
    pub port: u16,
    pub datasource_id: String,
    pub additional_config: T,
    pub scripting_api: Option<ScriptingApi>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ScriptingApi {
    pub url: String,
    pub token: String,
}

pub fn write_nvs_config<T>(nvs: &mut EspNvs<NvsDefault>, config: &Config<T>) -> Result<()>
where
    T: Serialize + Clone + Default,
{
    let json = serde_json::to_vec(config).map_err(|_| Error::SerializationError)?;
    nvs.set_raw(NVS_CONFIG_KEY, &json).map_err(Error::Esp)?;
    Ok(())
}

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
