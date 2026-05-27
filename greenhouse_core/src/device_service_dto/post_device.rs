use serde::{Deserialize, Serialize};

/// Request body for registering a new smart device.
#[derive(Serialize, Deserialize, Debug)]
pub struct PostDeviceDtoRequest {
    /// Human-readable name for the device.
    pub name: String,
    /// Human-readable description of the device's purpose.
    pub description: String,
    /// Network address at which the device HTTP server is reachable.
    pub address: String,
    /// Whether the device supports scripting automation.
    pub can_script: bool,
    /// Whether the device service should periodically scrape this device for readings.
    pub scraping: bool,
}
