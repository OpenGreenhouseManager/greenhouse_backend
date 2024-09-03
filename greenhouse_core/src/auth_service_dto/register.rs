use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequestDto {
    pub username: String,
    pub password: String,
    pub one_time_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponseDto {
    pub token: String,
    pub token_type: String,
}
