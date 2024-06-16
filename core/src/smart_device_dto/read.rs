use serde::{ Deserialize, Serialize };

use super::Type;

#[derive(Serialize, Deserialize, Default)]
pub struct ReadResponseDto {
    pub data: String,
    pub output_type: Type,
}
