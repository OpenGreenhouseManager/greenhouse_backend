use serde::{ Deserialize, Serialize };
pub mod endpoints;
pub mod read;
pub mod write;
pub mod config;
pub mod status;
#[derive(Serialize, Deserialize, Debug, Default)]
pub enum Type {
    Number,
    String,
    Stream,
    #[default]
    Unknown,
}
