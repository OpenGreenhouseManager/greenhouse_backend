use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

/// Importance level of an alert.
///
/// # Example
///
/// ```
/// # use greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity;
/// let s = Severity::Warning;
/// let json = serde_json::to_string(&s).unwrap();
/// assert_eq!(json, r#""Warning""#);
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub enum Severity {
    /// Informational — no action required.
    Info,
    /// Potential issue — warrants monitoring.
    Warning,
    /// Active problem — action may be required.
    Error,
    /// Critical failure — immediate action required.
    Fatal,
}

/// A single stored alert.
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct AlertDto {
    /// Unique identifier (UUID).
    pub id: String,
    /// Importance level of the alert.
    pub severity: Severity,
    /// Machine-readable event identifier (e.g. `"temperature_high"`).
    pub identifier: String,
    /// The sensor reading or value that triggered the alert.
    pub value: String,
    /// Optional human-readable note attached to the alert.
    pub note: Option<String>,
    /// ISO 8601 timestamp of when the alert was created.
    pub created_at: String,
    /// UUID of the data source (device) that raised the alert.
    pub datasource_id: String,
}

/// A paginated collection of [`AlertDto`] values.
///
/// # Example
///
/// ```
/// # use greenhouse_core::data_storage_service_dto::alert_dto::alert::{AlertDto, AlertsDto, Severity};
/// let alerts: AlertsDto = vec![
///     AlertDto {
///         id: "abc".to_string(),
///         severity: Severity::Info,
///         identifier: "test".to_string(),
///         value: "42".to_string(),
///         note: None,
///         created_at: "2024-01-01T00:00:00Z".to_string(),
///         datasource_id: "ds1".to_string(),
///     },
/// ]
/// .into();
/// assert_eq!(alerts.alerts.len(), 1);
/// ```
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct AlertsDto {
    /// The list of alerts.
    pub alerts: Vec<AlertDto>,
}

impl From<Vec<AlertDto>> for AlertsDto {
    fn from(alerts: Vec<AlertDto>) -> Self {
        Self { alerts }
    }
}
