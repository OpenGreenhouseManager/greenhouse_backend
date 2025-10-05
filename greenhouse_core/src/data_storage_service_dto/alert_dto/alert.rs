use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct AlertDto {
    pub id: String,
    pub severity: Severity,
    pub identifier: String,
    pub value: String,
    pub note: Option<String>,
    pub created_at: String,
    pub datasource_id: String,
}

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct AlertsDto {
    pub alerts: Vec<AlertDto>,
}

impl From<Vec<AlertDto>> for AlertsDto {
    fn from(alerts: Vec<AlertDto>) -> Self {
        Self { alerts }
    }
}
