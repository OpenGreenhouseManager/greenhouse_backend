use serde::{Deserialize, Serialize};

/// The operational status of a smart device.
#[derive(Serialize, Deserialize, Debug)]
pub enum DeviceStatusDto {
    /// Device is running and responding normally.
    Online,
    /// Device encountered an unrecoverable error and requires attention.
    Panic,
}

/// Response body returned by the device `/status` endpoint.
#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceStatusResponseDto {
    /// Current operational status.
    pub status: DeviceStatusDto,
    /// UUID of the data source associated with this device.
    pub datasource_id: String,
}
