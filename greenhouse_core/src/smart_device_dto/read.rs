use serde::{Deserialize, Serialize};

use super::Type;

/// Response body returned by the device `/read` endpoint.
///
/// Wraps the current sensor reading in a [`Type`] value so the caller
/// can determine both the data shape and the measurement.
#[derive(Serialize, Deserialize)]
pub struct ReadResponseDto {
    /// The current reading from the device.
    pub data: Type,
}
