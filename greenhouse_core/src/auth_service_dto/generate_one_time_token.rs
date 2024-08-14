use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateOneTimeTokenRequestDto {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateOneTimeTokenResponseDto {
    pub token: u64,
}
