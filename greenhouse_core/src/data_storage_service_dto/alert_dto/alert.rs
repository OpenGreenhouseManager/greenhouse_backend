use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlertDto {
    pub id: String,
    pub severity: Severity,
    pub identifier: String,
    pub value: String,
    pub note: Option<String>,
    pub created_at: String,
    pub datasource_id: String,
}
