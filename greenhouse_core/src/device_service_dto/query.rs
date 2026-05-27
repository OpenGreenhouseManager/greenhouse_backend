use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Query parameters for retrieving time-series data from a device.
///
/// Passed as URL query parameters to the timeseries endpoint. The `step`
/// parameter controls Prometheus-style resolution.
#[derive(Deserialize, Serialize, Debug)]
pub struct PromQuery {
    /// Start of the query time range (inclusive).
    pub start: DateTime<Utc>,
    /// End of the query time range (inclusive).
    pub end: DateTime<Utc>,
    /// Optional dot-notation path to extract a sub-property from Object readings.
    pub sub_property: Option<String>,
    /// Optional Prometheus duration string for down-sampling (e.g. `"1m"`, `"5s"`).
    pub step: Option<String>,
}
