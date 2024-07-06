use super::{Error, Result};
use crate::smart_device_dto::{self, config::ConfigResponseDto};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "config.json";

pub fn update_config_file<T>(config: &Config<T>) -> Result<()>
where
    T: Serialize + Clone + Default,
{
    let json_string = match serde_json::to_string(&config) {
        Ok(json_string) => json_string,
        Err(_) => return Err(Error::IllFormatedConfig),
    };
    match std::fs::write(CONFIG_FILE_NAME, json_string) {
        Ok(_) => (),
        Err(_) => return Err(Error::MissingConfig),
    };

    Ok(())
}

pub fn read_config_file<T>() -> Result<Config<T>>
where
    T: DeserializeOwned + Clone + Default,
{
    let data = match std::fs::read_to_string(CONFIG_FILE_NAME) {
        Ok(data) => data,
        Err(_) => return Err(Error::MissingConfig),
    };
    let a: Config<T> = match serde_json::from_str(&data) {
        Ok(a) => a,
        Err(_) => return Err(Error::IllFormatedConfig),
    };

    Ok(a)
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
    pub input_type: Option<Type>,
    pub output_type: Option<Type>,
    pub additinal_config: T,
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
            additinal_config: config.additinal_config,
        }
    }
}
