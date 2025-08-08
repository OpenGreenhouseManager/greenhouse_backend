use greenhouse_core::{
    data_storage_service_dto::alert_dto::{
        alert::AlertDto, endpoints, post_create_alert::CreateAlertDto,
    },
    http_error::ErrorResponseBody,
};

use crate::{
    alert::{Error, Result},
    helper::error::ApiError,
};
pub(crate) async fn create_alert(base_ulr: &str, alert: CreateAlertDto) -> Result<AlertDto> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::ALERT)
        .json(&alert)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!("Error in post to service: {:?} for url {}", e, base_ulr);

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!("Error in post to service: {:?} for url {}", e, base_ulr);

            Error::Json(e)
        });
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
