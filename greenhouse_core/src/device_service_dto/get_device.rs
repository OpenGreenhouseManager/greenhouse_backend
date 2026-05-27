use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

/// A registered smart device returned by the device registry.
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct DeviceResponseDto {
    /// Unique identifier (UUID) of the device.
    pub id: String,
    /// Human-readable name of the device.
    pub name: String,
    /// Network address (e.g. `"http://192.168.1.10:8080"`).
    pub address: String,
    /// Human-readable description of the device's purpose.
    pub description: String,
    /// Whether the device supports scripting automation. Wire-format name: `canscript`.
    pub canscript: bool,
    /// Whether the device service should periodically scrape this device for readings.
    pub scraping: bool,
}

/// A collection of registered devices.
///
/// # Example
///
/// ```
/// # use greenhouse_core::device_service_dto::get_device::{DeviceResponseDto, DevicesResponseDto};
/// let devices: DevicesResponseDto = vec![
///     DeviceResponseDto {
///         id: "uuid-1".to_string(),
///         name: "Temp Sensor".to_string(),
///         address: "http://192.168.1.10:8080".to_string(),
///         description: "Greenhouse temperature sensor".to_string(),
///         canscript: true,
///         scraping: true,
///     }
/// ]
/// .into();
/// assert_eq!(devices.devices.len(), 1);
/// ```
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct DevicesResponseDto {
    /// The list of registered devices.
    pub devices: Vec<DeviceResponseDto>,
}

impl From<Vec<DeviceResponseDto>> for DevicesResponseDto {
    fn from(devices: Vec<DeviceResponseDto>) -> Self {
        Self { devices }
    }
}
