use serde::{ de::DeserializeOwned, Deserialize, Serialize };

use crate::smart_device_dto::{ self, config::ConfigResponseDto };

const CONFIG_FILE_NAME: &str = "config.json";

pub fn update_config_file<T>(config: &Config<T>) -> Result<(), Box<dyn std::error::Error>>
    where T: Serialize + Clone + Default
{
    let data = serde_json::to_string(&config)?;
    std::fs::write(CONFIG_FILE_NAME, data)?;
    Ok(())
}

pub fn read_config_file<T>() -> Result<Config<T>, Box<dyn std::error::Error>>
    where T: DeserializeOwned + Clone + Default
{
    let data = std::fs::read_to_string(CONFIG_FILE_NAME)?;
    let a: Config<T> = serde_json::from_str(&data)?;
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

impl Into<smart_device_dto::Type> for Type {
    fn into(self) -> smart_device_dto::Type {
        match self {
            Type::Number => smart_device_dto::Type::Number,
            Type::String => smart_device_dto::Type::String,
            Type::Stream => smart_device_dto::Type::Stream,
            Type::Unknown => smart_device_dto::Type::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config<T> where T: Clone + Default {
    pub mode: Mode,
    pub input_type: Option<Type>,
    pub output_type: Option<Type>,
    pub additinal_config: T,
}

impl<T> From<Config<T>> for ConfigResponseDto<T> where T: Clone + Default {
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
