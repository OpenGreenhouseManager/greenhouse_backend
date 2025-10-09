use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, IntoJsonResponse)]
pub struct OperationsDto {
    pub operations: Vec<String>,
}

impl From<Vec<String>> for OperationsDto {
    fn from(operations: Vec<String>) -> Self {
        Self { operations }
    }
}
