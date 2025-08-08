use crate::data_storage_service_dto::alert_dto::alert::Severity;
use crate::data_storage_service_dto::alert_dto::post_create_alert::CreateAlertDto;
use std::sync::Arc;

use super::config::Config;
use super::{Error, Result};

pub struct AlertCreation {
    pub severity: Severity,
    pub identifier: String,
    pub value: Option<String>,
    pub note: Option<String>,
}

pub async fn trigger_alert<T>(config: Arc<Config<T>>, alert: AlertCreation) -> Result<()>
where
    T: Clone + Default,
{
    if let Some(scripting_api) = &config.scripting_api {
        let client = reqwest::Client::new();

        let alert = CreateAlertDto {
            severity: alert.severity,
            identifier: alert.identifier,
            value: alert.value,
            note: alert.note,
            datasource_id: config.datasource_id.clone(),
        };

        let response = client
            .post(format!("{}/alert", scripting_api.url))
            .json(&alert)
            .header("Access-Control-Allow-Credentials", "true")
            .header("Cookie", format!("auth-token={}", scripting_api.token))
            .send()
            .await
            .map_err(Error::Request)?;

        if !response.status().is_success() {
            return Err(Error::Request(response.error_for_status().unwrap_err()));
        }
    }
    Ok(())
}
