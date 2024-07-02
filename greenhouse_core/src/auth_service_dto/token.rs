use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenRequestDto {
    pub token: String,
    pub token_type: String,
}
