use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AlertAggrigatedDto {
    pub count: u32,
    pub identifier: String,
    pub source: SourceDto,
    pub latest_value: String,
    pub first: String,
    pub last: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceDto {
    pub id: String,
    pub source_type: String,
}
