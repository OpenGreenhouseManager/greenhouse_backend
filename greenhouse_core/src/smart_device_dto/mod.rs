use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod activation;
pub mod config;
pub mod endpoints;
pub mod read;
pub mod status;
pub mod write;

#[derive(Serialize, Deserialize)]
pub enum Type {
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, Type>),
    Measurement(Measurement),
    Stream,
    None,
}

#[derive(Serialize, Deserialize)]
pub struct Measurement {
    pub value: f64,
    pub unit: String,
}
