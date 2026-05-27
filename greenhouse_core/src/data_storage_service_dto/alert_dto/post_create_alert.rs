use serde::{Deserialize, Serialize};

use super::alert::Severity;

/// Request body for creating a new alert.
///
/// Sent by smart devices (via `trigger_alert`) or by automation scripts to
/// record a notable event in the data storage service.
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAlertDto {
    /// Importance level of the new alert.
    pub severity: Severity,
    /// Machine-readable event identifier (e.g. `"temperature_high"`).
    pub identifier: String,
    /// The sensor reading or value that triggered the alert, if available.
    pub value: Option<String>,
    /// Optional human-readable note.
    pub note: Option<String>,
    /// UUID of the data source (device) raising the alert.
    pub datasource_id: String,
}
