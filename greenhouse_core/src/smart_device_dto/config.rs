use serde::{Deserialize, Serialize};

use crate::smart_device_dto::TypeOption;

#[derive(Serialize, Deserialize)]
pub struct ConfigResponseDto<T> {
    pub mode: Mode,
    pub input_type: Option<TypeOption>,
    pub output_type: Option<TypeOption>,
    pub scripting_api: Option<ScriptingApi>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScriptingApi {
    pub url: String,
    pub token: String,
}
