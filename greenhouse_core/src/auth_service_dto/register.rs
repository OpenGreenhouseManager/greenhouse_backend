use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequestDto {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponseDto {
    pub token: String,
    pub token_type: String,
}
