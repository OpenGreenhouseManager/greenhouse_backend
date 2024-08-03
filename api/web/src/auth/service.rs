use crate::auth::{Error, Result};

use greenhouse_core::auth_service_dto::{
    endpoints,
    login::{LoginRequestDto, LoginResponseDto},
    register::{RegisterRequestDto, RegisterResponseDto},
    token::TokenRequestDto,
};

pub async fn register(base_ulr: &str, user: RegisterRequestDto) -> Result<RegisterResponseDto> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::REGISTER)
        .json(&user)
        .send()
        .await
        .map_err(|_| Error::InternalError)?;
    resp.json().await.map_err(|_| Error::InternalError)
}

pub async fn login(base_ulr: &str, user: LoginRequestDto) -> Result<LoginResponseDto> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::LOGIN)
        .json(&user)
        .send()
        .await
        .map_err(|_| Error::InternalError)?;
    resp.json().await.map_err(|_| Error::InternalError)
}

pub async fn check_token(base_ulr: &str, token: &str) -> Result<()> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::CHECK_TOKEN)
        .json(&TokenRequestDto {
            token: token.to_string(),
            token_type: String::from("Bearer"),
        })
        .send()
        .await
        .map_err(|_| Error::InternalError)?;
    if resp.status().is_success() {
        return Ok(());
    }
    Err(Error::Unauthorized)
}
