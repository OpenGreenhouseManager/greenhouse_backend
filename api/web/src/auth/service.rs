use crate::auth::{Error, Result};

use greenhouse_core::auth_service_dto::{
    endpoints,
    login::{LoginRequestDto, LoginResponseDto},
    register::{RegisterRequestDto, RegisterResponseDto},
    token::TokenRequestDto,
};

pub async fn register(
    base_ulr: &str,
    register_request: RegisterRequestDto,
) -> Result<RegisterResponseDto> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::REGISTER)
        .json(&register_request)
        .send()
        .await
        .map_err(|_| Error::InternalError)?;
    resp.json().await.map_err(|_| Error::InternalError)
}

pub async fn login(base_ulr: &str, login_request: LoginRequestDto) -> Result<LoginResponseDto> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::LOGIN)
        .json(&login_request)
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
