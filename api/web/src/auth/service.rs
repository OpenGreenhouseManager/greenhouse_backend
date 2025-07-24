use crate::{
    auth::{Error, Result},
    helper::error::ApiError,
};

use greenhouse_core::auth_service_dto::{
    endpoints,
    login::{LoginRequestDto, LoginResponseDto},
    register::{RegisterRequestDto, RegisterResponseDto},
    token::TokenRequestDto,
};

pub(crate) async fn register(
    base_ulr: &str,
    register_request: RegisterRequestDto,
) -> Result<RegisterResponseDto> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::REGISTER)
        .json(&register_request)
        .send()
        .await
        .map_err(|e| {
            sentry::configure_scope(|scope| {
                let mut map = std::collections::BTreeMap::new();
                map.insert(
                    String::from("username"),
                    register_request.username.clone().into(),
                );

                scope.set_context("username", sentry::protocol::Context::Other(map));
            });

            tracing::error!(
                "Error in post to service: {:?} with username: {} for url {}",
                e,
                register_request.username,
                base_ulr
            );

            sentry::capture_error(&e);
            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::configure_scope(|scope| {
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), register_request.username.into());

                scope.set_context("username", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);

            tracing::error!("Error in response json: {:?}", e,);

            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp.text().await.unwrap_or_default(),
    }))
}

pub(crate) async fn login(
    base_ulr: &str,
    login_request: LoginRequestDto,
) -> Result<LoginResponseDto> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::LOGIN)
        .json(&login_request)
        .send()
        .await
        .map_err(|e| {
            sentry::configure_scope(|scope| {
                let mut map = std::collections::BTreeMap::new();
                map.insert(
                    String::from("username"),
                    login_request.username.clone().into(),
                );

                scope.set_context("username", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);

            tracing::error!(
                "Error in post to service: {:?} with username: {} for url {}",
                e,
                login_request.username,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_extra("username", login_request.username.into());
            });
            sentry::capture_error(&e);

            tracing::error!("Error in response json: {:?}", e,);

            Error::Json(e)
        });
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp.text().await.unwrap_or_default(),
    }))
}

pub(crate) async fn check_token(base_ulr: &str, token: &str) -> Result<()> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::CHECK_TOKEN)
        .json(&TokenRequestDto {
            token: token.to_string(),
            token_type: String::from("Bearer"),
        })
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in post to service: {:?} with token: {} for url {}",
                e,
                token,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return Ok(());
    }
    Err(Error::Api(ApiError {
        status: resp.status(),
        message: resp.text().await.unwrap_or_default(),
    }))
}
