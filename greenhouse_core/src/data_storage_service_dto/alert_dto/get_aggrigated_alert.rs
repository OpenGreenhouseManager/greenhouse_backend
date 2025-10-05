use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

use super::alert::Severity;

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct AggrigatedAlertDto {
    pub count: i64,
    pub identifier: String,
    pub severity: Severity,
    pub source: String,
    pub first: String,
    pub last: String,
}

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct AgrigatedAlertsDto {
    pub alerts: Vec<AggrigatedAlertDto>,
}

impl From<Vec<AggrigatedAlertDto>> for AgrigatedAlertsDto {
    fn from(alerts: Vec<AggrigatedAlertDto>) -> Self {
        Self { alerts }
    }
}
