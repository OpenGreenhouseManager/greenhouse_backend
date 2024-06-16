use serde::{ Deserialize, Serialize };

use super::Type;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigResponseDto<T> {
    pub address: String,
    pub port: u32,
    pub mode: Mode,
    pub input_type: Option<Type>,
    pub output_type: Option<Type>,
    pub additinal_config: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Mode {
    Input,
    Output,
    InputOutput,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigRequestDto<T> {
    pub address: String,
    pub port: u32,
    pub additinal_config: T,
}
