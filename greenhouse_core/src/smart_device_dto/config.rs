use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigResponseDto<T> {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TypeOption {
    Number,
    Boolean,
    Object,
    Array,
    Stream,
    Unknown,
}
