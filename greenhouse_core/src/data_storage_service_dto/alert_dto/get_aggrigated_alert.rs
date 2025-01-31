use serde::{Deserialize, Serialize};

use super::alert::Severity;

#[derive(Serialize, Deserialize, Debug)]
pub struct AlertAggrigatedDto {
    pub count: i64,
    pub identifier: String,
    pub severity: Severity,
    pub source: String,
    pub first: String,
    pub last: String,
}
