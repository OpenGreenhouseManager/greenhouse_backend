use serde::{Deserialize, Serialize};

/// Request body for updating an existing smart device registration.
///
/// All fields are required; partial updates are not supported.
#[derive(Serialize, Deserialize, Debug)]
pub struct PutDeviceDtoRequest {
    /// Updated human-readable name.
    pub name: String,
    /// Updated description.
    pub description: String,
    /// Updated network address.
    pub address: String,
    /// Updated scripting support flag.
    pub can_script: bool,
    /// Updated scraping flag.
    pub scraping: bool,
}
