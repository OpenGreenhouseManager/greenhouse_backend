use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::alert::Severity;

#[derive(Deserialize, Serialize, Debug)]
pub struct AlertQuery {
    pub severity: Option<Severity>,
    pub identifier: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub datasource_id: Option<Uuid>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IntervalQuery {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}
