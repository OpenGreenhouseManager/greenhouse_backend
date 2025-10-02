use greenhouse_macro::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, IntoResponse)]
pub struct DeviceResponseDto {
    pub id: String,
    pub name: String,
    pub address: String,
    pub description: String,
    pub canscript: bool,
    pub scraping: bool,
}
