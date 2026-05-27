use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

/// The list of operations (endpoints) supported by a smart device.
///
/// Returned by the device service to inform clients which HTTP endpoints
/// a given device exposes (e.g. `["read", "status", "config"]`).
#[derive(Serialize, Deserialize, IntoJsonResponse)]
pub struct OperationsDto {
    /// Names of the supported operations.
    pub operations: Vec<String>,
}

impl From<Vec<String>> for OperationsDto {
    fn from(operations: Vec<String>) -> Self {
        Self { operations }
    }
}
