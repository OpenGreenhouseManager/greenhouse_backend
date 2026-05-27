use std::collections::HashMap;

use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

/// Time-series sensor readings for a device, returned by the timeseries endpoint.
#[derive(Serialize, Deserialize, IntoJsonResponse)]
pub struct GetTimeseriesDto {
    /// Ordered list of timestamped readings.
    pub timeseries: Vec<TimeseriesDto>,
}

/// A single timestamped sensor reading.
#[derive(Serialize, Deserialize)]
pub struct TimeseriesDto {
    /// Unix timestamp (milliseconds) of the reading.
    pub timestamp: u64,
    /// The sensor value at this point in time.
    pub value: Type,
}

/// The value type for a stored time-series reading.
///
/// This enum mirrors [`smart_device_dto::Type`](crate::smart_device_dto::Type)
/// but excludes `Stream` and `None` since those cannot be stored as discrete values.
#[derive(Serialize, Deserialize)]
pub enum Type {
    /// A numeric sensor reading (e.g. raw ADC counts, voltage).
    Number(f64),
    /// A boolean sensor state (e.g. relay on/off).
    Boolean(bool),
    /// A structured key-value reading (e.g. multi-channel sensor).
    Object(HashMap<String, Type>),
    /// A physical measurement with a value and a unit string.
    Measurement(Measurement),
}

/// A physical measurement pairing a numeric value with its unit.
#[derive(Serialize, Deserialize)]
pub struct Measurement {
    /// The measured quantity.
    pub value: f64,
    /// SI or custom unit string (e.g. `"°C"`, `"hPa"`, `"%RH"`).
    pub unit: String,
}

impl From<Vec<TimeseriesDto>> for GetTimeseriesDto {
    fn from(timeseries: Vec<TimeseriesDto>) -> Self {
        Self { timeseries }
    }
}
