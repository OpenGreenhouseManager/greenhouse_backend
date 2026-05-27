use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::alert::Severity;

/// Query parameters for filtering stored alerts.
///
/// All fields are optional; supply only the filters you need.
#[derive(Deserialize, Serialize, Debug)]
pub struct AlertQuery {
    /// Restrict results to alerts of this severity level.
    pub severity: Option<Severity>,
    /// Restrict results to alerts with this machine-readable identifier.
    pub identifier: Option<String>,
    /// Return only alerts created at or after this timestamp.
    pub created_at: Option<DateTime<Utc>>,
    /// Restrict results to alerts from this data source UUID.
    pub datasource_id: Option<Uuid>,
}

/// Time-range query parameters for interval-based alert queries.
#[derive(Deserialize, Serialize, Debug)]
pub struct IntervalQuery {
    /// Start of the time range (inclusive). `None` means no lower bound.
    pub start: Option<DateTime<Utc>>,
    /// End of the time range (inclusive). `None` means no upper bound.
    pub end: Option<DateTime<Utc>>,
}
