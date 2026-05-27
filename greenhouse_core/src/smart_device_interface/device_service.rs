use crate::data_storage_service_dto::alert_dto::alert::Severity;
use crate::data_storage_service_dto::alert_dto::post_create_alert::CreateAlertDto;
use std::sync::Arc;

use super::config::Config;
use super::{Error, Result};

/// Data needed to raise an alert through the scripting service.
///
/// Pass this to [`trigger_alert`] after constructing it from sensor data or
/// error conditions detected in a handler function.
pub struct AlertCreation {
    /// Importance level of the alert.
    pub severity: Severity,
    /// Machine-readable event identifier (e.g. `"temperature_high"`).
    pub identifier: String,
    /// The sensor reading or value that triggered the alert, if available.
    pub value: Option<String>,
    /// Optional human-readable note for context.
    pub note: Option<String>,
}

/// Posts an alert to the scripting service.
///
/// The device must have been activated (i.e. `config.scripting_api` must be
/// `Some`) for this to succeed. If not, [`Error::ScriptingApiNotConfigured`]
/// is returned immediately without making any network request.
///
/// # Errors
///
/// - [`Error::ScriptingApiNotConfigured`] — the device has not been activated.
/// - [`Error::Request`] — the HTTP POST to the scripting service failed.
///
/// # Example
///
/// ```no_run
/// # use std::sync::Arc;
/// # use greenhouse_core::smart_device_interface::{
/// #     config::Config, device_service::{AlertCreation, trigger_alert},
/// # };
/// # use greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity;
/// # #[derive(Clone, Default)]
/// # struct MyCfg;
/// # async fn example(config: Arc<Config<MyCfg>>) {
/// trigger_alert(config, AlertCreation {
///     severity: Severity::Warning,
///     identifier: "temperature_high".to_string(),
///     value: Some("35.2".to_string()),
///     note: None,
/// })
/// .await
/// .unwrap();
/// # }
/// ```
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
        return Ok(());
    }
    Err(Error::ScriptingApiNotConfigured)
}
