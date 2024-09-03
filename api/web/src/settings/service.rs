use crate::settings::{Error, Result};

use greenhouse_core::auth_service_dto::{
    endpoints,
    generate_one_time_token::{GenerateOneTimeTokenRequestDto, GenerateOneTimeTokenResponseDto},
};

pub async fn generate_one_time_token(base_ulr: &str, username: &str) -> Result<String> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::GENERATE_ONE_TIME_TOKEN)
        .json(&GenerateOneTimeTokenRequestDto {
            username: String::from(username),
        })
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::RegisterToken
        })?;
    if resp.status().is_success() {
        return Ok(resp
            .json::<GenerateOneTimeTokenResponseDto>()
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                Error::RegisterToken
            })?
            .token);
    }
    Err(Error::RegisterToken)
}
