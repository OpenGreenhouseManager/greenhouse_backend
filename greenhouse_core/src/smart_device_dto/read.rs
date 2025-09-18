use serde::{Deserialize, Serialize};

use super::Type;

#[derive(Serialize, Deserialize)]
pub struct ReadResponseDto {
    pub data: Type,
}
