use serde::{ de::DeserializeOwned, Deserialize, Serialize };

use crate::smart_device_dto::config::ConfigResponseDto;

const CONFIG_FILE_NAME: &str = "config.json";

pub fn update_config_file<T>(config: &Config<T>) -> Result<(), Box<dyn std::error::Error>>
    where T: Serialize + Clone
{
    let data = serde_json::to_string(&config)?;
    std::fs::write(CONFIG_FILE_NAME, data)?;
    Ok(())
}

pub fn read_config_file<T>() -> Result<Config<T>, Box<dyn std::error::Error>>
    where T: DeserializeOwned + Clone
{
    let data = std::fs::read_to_string(CONFIG_FILE_NAME)?;
    let a: Config<T> = serde_json::from_str(&data)?;
    Ok(a)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Mode {
    Input,
    Output,
    InputOutput,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Type {
    Number,
    String,
    Stream,
    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config<T> where T: Clone {
    pub address: String,
    pub port: u32,
    pub mode: Mode,
    pub input_type: Option<Type>,
    pub output_type: Option<Type>,
    pub additinal_config: T,
}

impl<T> Into<ConfigResponseDto<T>> for Config<T> where T: Clone {
    fn into(self) -> ConfigResponseDto<T> {
        ConfigResponseDto {
            address: self.address,
            port: self.port,
            mode: match self.mode {
                Mode::Input => crate::smart_device_dto::config::Mode::Input,
                Mode::Output => crate::smart_device_dto::config::Mode::Output,
                Mode::InputOutput => crate::smart_device_dto::config::Mode::InputOutput,
            },
            input_type: match self.input_type {
                Some(Type::Number) => Some(crate::smart_device_dto::Type::Number),
                Some(Type::String) => Some(crate::smart_device_dto::Type::String),
                Some(Type::Stream) => Some(crate::smart_device_dto::Type::Stream),
                Some(Type::Unknown) => Some(crate::smart_device_dto::Type::Unknown),
                None => None,
            },
            output_type: match self.output_type {
                Some(Type::Number) => Some(crate::smart_device_dto::Type::Number),
                Some(Type::String) => Some(crate::smart_device_dto::Type::String),
                Some(Type::Stream) => Some(crate::smart_device_dto::Type::Stream),
                Some(Type::Unknown) => Some(crate::smart_device_dto::Type::Unknown),
                None => None,
            },
            additinal_config: self.additinal_config,
        }
    }
}
