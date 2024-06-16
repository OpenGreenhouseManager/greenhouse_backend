use serde::{Deserialize, Serialize};
pub mod config;
pub mod endpoints;
pub mod read;
pub mod status;
pub mod write;
#[derive(Serialize, Deserialize, Debug, Default)]
pub enum Type {
    Number,
    String,
    Stream,
    #[default]
    Unknown,
}
