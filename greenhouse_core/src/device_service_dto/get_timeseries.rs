use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetTimeseriesDto {
    pub timeseries: Vec<TimeseriesDto>,
}

#[derive(Serialize, Deserialize)]
pub struct TimeseriesDto {
    pub timestamp: u64,
    pub value: Type,
}

#[derive(Serialize, Deserialize)]
pub enum Type {
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, Type>),
    Array(Vec<Type>),
}
