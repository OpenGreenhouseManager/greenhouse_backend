use crate::{
    helper::error::ApiError,
    settings::{Error, Result},
};

use greenhouse_core::{
    auth_service_dto::{
        endpoints,
        generate_one_time_token::{
            GenerateOneTimeTokenRequestDto, GenerateOneTimeTokenResponseDto,
        },
    },
    http_error::ErrorResponseBody,
};

pub(crate) async fn generate_one_time_token(base_ulr: &str, username: &str) -> Result<String> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::GENERATE_ONE_TIME_TOKEN)
        .json(&GenerateOneTimeTokenRequestDto {
            username: String::from(username),
        })
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in post to service: {:?} with username: {} for url {}",
                e,
                username,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return Ok(resp
            .json::<GenerateOneTimeTokenResponseDto>()
            .await
            .map_err(|e| {
                sentry::capture_error(&e);

                tracing::error!("Error in response json: {:?}", e,);

                Error::Json(e)
            })?
            .token);
    }

    sentry::capture_message(
        &format!(
            "Error from service: {:?} for username {}",
            resp.status(),
            username
        ),
        sentry::Level::Error,
    );

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
