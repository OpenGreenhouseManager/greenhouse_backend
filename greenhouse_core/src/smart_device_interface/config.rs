use crate::smart_device_dto::config::TypeOption;

use super::{Error, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

// Default config file path for backward compatibility
pub(crate) const DEFAULT_CONFIG_FILE_NAME: &str = "./config/config.json";

pub fn update_config_file<T>(config: &Config<T>) -> Result<()>
where
    T: Serialize + Clone + Default,
{
    update_config_file_with_path(config, DEFAULT_CONFIG_FILE_NAME)
}

pub fn read_config_file<T>() -> Result<Config<T>>
where
    T: DeserializeOwned + Clone + Default,
{
    read_config_file_with_path(DEFAULT_CONFIG_FILE_NAME)
}

pub fn update_config_file_with_path<T>(config: &Config<T>, config_path: &str) -> Result<()>
where
    T: Serialize + Clone + Default,
{
    let json_string = serde_json::to_string(&config).map_err(|_| Error::IllFormattedConfig)?;
    std::fs::write(config_path, json_string).map_err(|_| Error::MissingConfig)
}

pub fn read_config_file_with_path<T>(config_path: &str) -> Result<Config<T>>
where
    T: DeserializeOwned + Clone + Default,
{
    let data = std::fs::read_to_string(config_path).map_err(|_| Error::MissingConfig)?;
    serde_json::from_str(&data).map_err(|_| Error::IllFormattedConfig)
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Mode {
    Input(TypeOption),
    Output(TypeOption),
    InputOutput(TypeOption, TypeOption),
    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub enum TypeOptionDto {
    Number,
    Boolean,
    Object,
    Measurement,
    Stream,
    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config<T>
where
    T: Clone + Default,
{
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
