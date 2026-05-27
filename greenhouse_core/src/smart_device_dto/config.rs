use serde::{Deserialize, Serialize};

/// Full device configuration returned by the `/config` GET endpoint.
///
/// Describes the device's operating mode, data type, scripting API connection,
/// and any device-specific additional configuration.
#[derive(Serialize, Deserialize)]
pub struct ConfigResponseDto<T> {
    /// Whether the device acts as input, output, or both.
    pub mode: Mode,
    /// The data type produced by this device (present when `mode` is `Input` or `InputOutput`).
    pub input_type: Option<TypeOption>,
    /// The data type accepted by this device (present when `mode` is `Output` or `InputOutput`).
    pub output_type: Option<TypeOption>,
    /// Scripting service connection details, if the device has been activated.
    pub scripting_api: Option<ScriptingApi>,
    /// Device-specific configuration fields beyond the standard ones.
    pub additional_config: T,
}

/// The operating mode of a smart device.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Mode {
    /// Sensor only — exposes the `/read` endpoint.
    Input,
    /// Actuator only — exposes the `/write` endpoint.
    Output,
    /// Both sensor and actuator — exposes `/read` and `/write`.
    InputOutput,
    /// Mode has not been determined yet.
    Unknown,
}

/// Request body for updating device-specific configuration via `/config` POST.
///
/// Only the `additional_config` portion of the configuration can be updated
/// through this endpoint; port, mode, and scripting API settings are managed
/// separately.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigRequestDto<T> {
    /// New values for device-specific configuration fields.
    pub additional_config: T,
}

/// Scripting service connection details stored in device configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScriptingApi {
    /// Base URL of the scripting service.
    pub url: String,
    /// Bearer token for scripting service authentication.
    pub token: String,
}

/// The data type that a device reads or accepts.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TypeOption {
    /// A raw numeric value (`f64`).
    Number,
    /// A boolean state.
    Boolean,
    /// A structured key-value object.
    Object,
    /// A physical measurement with value and unit.
    Measurement,
    /// A data stream (no discrete value).
    Stream,
    /// Type is not yet known.
    Unknown,
}
