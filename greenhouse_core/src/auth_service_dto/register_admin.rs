use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterAdminRequestDto {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterAdminResponseDto {
    pub token: String,
    pub token_type: String,
}
