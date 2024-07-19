use serde::{Deserialize, Serialize};

use super::Type;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigResponseDto<T> {
    pub mode: Mode,
    pub input_type: Option<Type>,
    pub output_type: Option<Type>,
    pub additional_config: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Mode {
    Input,
    Output,
    InputOutput,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigRequestDto<T> {
    pub additional_config: T,
}
