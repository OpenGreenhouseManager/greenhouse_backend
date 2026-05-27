use serde::{Deserialize, Serialize};

use crate::smart_device_dto::Type;

/// Request body sent to the device `/write` endpoint.
///
/// Used to command an actuator, set a relay state, or push a value to a
/// device that accepts external input.
#[derive(Serialize, Deserialize)]
pub struct WriteRequestDto {
    /// The value to write to the device.
    pub data: Type,
}
