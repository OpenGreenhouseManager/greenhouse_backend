use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AlertAggrigatedDto {
    pub count: i64,
    pub identifier: String,
    pub source: String,
    pub latest_value: String,
    pub first: String,
    pub last: String,
}
