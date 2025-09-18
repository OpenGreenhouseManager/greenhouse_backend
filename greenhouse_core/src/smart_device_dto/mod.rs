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
    String(String),
    Boolean(bool),
    Object(HashMap<String, Type>),
    Array(Vec<Type>),
    Stream,
    Unknown(String),
}
