use greenhouse_core::{
    data_storage_service_dto::alert_dto::{
        alert::AlertsDto,
        endpoints,
        get_aggrigated_alert::AgrigatedAlertsDto,
        query::{AlertQuery, IntervalQuery},
    },
    http_error::ErrorResponseBody,
};

use crate::{
    alert::{Error, Result},
    helper::error::ApiError,
};

pub(crate) async fn get_filtered_alert(base_ulr: &str, query: AlertQuery) -> Result<AlertsDto> {
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string() + endpoints::ALERT + "/filter")
        .query(&query)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in get to service: {:?} with id: {:?} for url {}",
                e,
                query,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!("Error in get to service: {:?} with id: {:?}", e, query);

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

pub(crate) async fn get_alert_subset(
    base_ulr: &str,
    query: IntervalQuery,
) -> Result<AgrigatedAlertsDto> {
    let resp = reqwest::Client::new()
        .get(base_ulr.to_string() + endpoints::ALERT)
        .query(&query)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!(
                "Error in get to service: {:?} with id: {:?} for url {}",
                e,
                query,
                base_ulr
            );

            Error::Request(e)
        })?;
    if resp.status().is_success() {
        return resp.json().await.map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!("Error in get to service: {:?} with id: {:?}", e, query);

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
