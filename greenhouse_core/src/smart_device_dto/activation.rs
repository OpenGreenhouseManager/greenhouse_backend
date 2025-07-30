use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActivateRequestDto {
    pub url: String,
    pub token: String,
}