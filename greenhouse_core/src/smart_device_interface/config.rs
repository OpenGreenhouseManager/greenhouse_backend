use super::{Error, Result};
use crate::smart_device_dto::{self, config::ConfigResponseDto};
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
    Input,
    Output,
    InputOutput,
    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub enum Type {
    Number,
    String,
    Stream,
    #[default]
    Unknown,
}

impl From<Type> for smart_device_dto::Type {
    fn from(val: Type) -> Self {
        match val {
            Type::Number => smart_device_dto::Type::Number,
            Type::String => smart_device_dto::Type::String,
            Type::Stream => smart_device_dto::Type::Stream,
            Type::Unknown => smart_device_dto::Type::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config<T>
where
    T: Clone + Default,
{
    pub mode: Mode,
    pub port: u16,
    pub input_type: Option<Type>,
    pub output_type: Option<Type>,
    pub additional_config: T,
    pub scripting_api: Option<ScriptingApi>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ScriptingApi {
    pub url: String,
    pub token: String,
}

impl<T> From<Config<T>> for ConfigResponseDto<T>
where
    T: Clone + Default,
{
    fn from(config: Config<T>) -> Self {
        ConfigResponseDto {
            mode: match config.mode {
                Mode::Input => crate::smart_device_dto::config::Mode::Input,
                Mode::Output => crate::smart_device_dto::config::Mode::Output,
                Mode::InputOutput => crate::smart_device_dto::config::Mode::InputOutput,
                Mode::Unknown => crate::smart_device_dto::config::Mode::Unknown,
            },
            input_type: match config.input_type {
                Some(Type::Number) => Some(crate::smart_device_dto::Type::Number),
                Some(Type::String) => Some(crate::smart_device_dto::Type::String),
                Some(Type::Stream) => Some(crate::smart_device_dto::Type::Stream),
                Some(Type::Unknown) => Some(crate::smart_device_dto::Type::Unknown),
                None => None,
            },
            output_type: match config.output_type {
                Some(Type::Number) => Some(crate::smart_device_dto::Type::Number),
                Some(Type::String) => Some(crate::smart_device_dto::Type::String),
                Some(Type::Stream) => Some(crate::smart_device_dto::Type::Stream),
                Some(Type::Unknown) => Some(crate::smart_device_dto::Type::Unknown),
                None => None,
            },
            scripting_api: config.scripting_api.map(|s| {
                crate::smart_device_dto::config::ScriptingApi {
                    url: s.url,
                    token: s.token,
                }
            }),
            additional_config: config.additional_config,
        }
    }
}
