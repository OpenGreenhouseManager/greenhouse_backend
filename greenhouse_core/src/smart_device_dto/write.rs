use serde::{Deserialize, Serialize};

use crate::smart_device_dto::Type;

#[derive(Serialize, Deserialize)]
pub struct WriteRequestDto {
    pub data: Type,
}
