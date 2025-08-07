use crate::{
    auth::{Error, Result},
    helper::error::ApiError,
};

use greenhouse_core::{
    scripting_service_dto::{endpoints, token::TokenDto},
    http_error::ErrorResponseBody,
};

pub(crate) async fn check_token(base_ulr: &str, token: &str) -> Result<()> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::CHECK_TOKEN)
        .json(&TokenDto {
            token: token.to_string(),
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
        message: resp
            .json::<ErrorResponseBody>()
            .await
            .map_err(|e| {
                sentry::capture_error(&e);
                tracing::error!("Error in get to service: {:?}", e);
                Error::Json(e)
            })?
            .error,
    }))
}
