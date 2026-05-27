use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

use super::alert::Severity;

/// Aggregated summary of alerts sharing the same identifier.
///
/// Instead of returning every individual alert, the API can group them by
/// `identifier` and return one `AggrigatedAlertDto` per group, showing the
/// total count and the first/last occurrence timestamps.
///
/// > **Note:** The identifier `AggrigatedAlertDto` preserves the original
/// > wire-format name used in the API. Use `count`, `first`, and `last` to
/// > determine the frequency and recency of the alert group.
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct AggrigatedAlertDto {
    /// Number of alerts in this group.
    pub count: i64,
    /// Machine-readable event identifier shared by all alerts in the group.
    pub identifier: String,
    /// Highest severity level observed within the group.
    pub severity: Severity,
    /// UUID of the data source (device) that raised the alerts.
    pub source: String,
    /// ISO 8601 timestamp of the earliest alert in the group.
    pub first: String,
    /// ISO 8601 timestamp of the most recent alert in the group.
    pub last: String,
}

/// A collection of [`AggrigatedAlertDto`] values.
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct AggrigatedAlertsDto {
    /// The list of aggregated alert groups.
    pub alerts: Vec<AggrigatedAlertDto>,
}

impl From<Vec<AggrigatedAlertDto>> for AggrigatedAlertsDto {
    fn from(alerts: Vec<AggrigatedAlertDto>) -> Self {
        Self { alerts }
    }
}
