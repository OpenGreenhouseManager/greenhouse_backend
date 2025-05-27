use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::AlertDto,
    endpoints,
    get_aggrigated_alert::AlertAggrigatedDto,
    post_create_alert::CreateAlertDto,
    query::{AlertQuery, IntervalQuery},
};

use crate::alert::{Error, Result};

pub(crate) async fn get_filtered_alert(base_ulr: &str, query: AlertQuery) -> Result<Vec<AlertDto>> {
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

            Error::InternalError
        })?;
    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);

        tracing::error!("Error in get to service: {:?} with id: {:?}", e, query);

        Error::InternalError
    })
}

pub(crate) async fn get_alert_subset(
    base_ulr: &str,
    query: IntervalQuery,
) -> Result<Vec<AlertAggrigatedDto>> {
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

            Error::InternalError
        })?;
    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);

        tracing::error!("Error in get to service: {:?} with id: {:?}", e, query);

        Error::InternalError
    })
}

pub(crate) async fn create_alert(base_ulr: &str, alert: CreateAlertDto) -> Result<AlertDto> {
    let resp = reqwest::Client::new()
        .post(base_ulr.to_string() + endpoints::ALERT)
        .json(&alert)
        .send()
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            tracing::error!("Error in post to service: {:?} for url {}", e, base_ulr);

            Error::InternalError
        })?;
    resp.json().await.map_err(|e| {
        sentry::capture_error(&e);

        tracing::error!("Error in post to service: {:?} for url {}", e, base_ulr);

        Error::InternalError
    })
}
