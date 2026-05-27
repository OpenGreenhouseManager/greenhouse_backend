use std::sync::Arc;

use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::Severity, post_create_alert::CreateAlertDto,
};

use crate::config::Config;
use crate::error::{Error, Result};

/// Data needed to raise an alert through the scripting service.
///
/// Pass this to [`trigger_alert`] when a sensor threshold is crossed or a
/// device fault is detected.
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

/// Posts an alert to the scripting service using a blocking HTTP client.
///
/// Unlike the Linux version, this function is synchronous and uses the
/// ESP-IDF HTTP client via `esp-idf-svc`.
///
/// # Errors
///
/// - [`Error::ScriptingApiNotConfigured`] — the device has not been activated.
/// - [`Error::SerializationError`] — the alert DTO could not be serialised.
/// - [`Error::IoError`] — the HTTP request could not be sent.
/// - [`Error::HttpStatus`] — the scripting service returned a non-2xx status.
pub fn trigger_alert<T>(config: &Arc<Config<T>>, alert: AlertCreation) -> Result<()>
where
    T: Clone + Default,
{
    let scripting_api = config.scripting_api.as_ref().ok_or_else(|| {
        log::warn!("scripting_api missing; cannot send alert");
        Error::ScriptingApiNotConfigured
    })?;

    let dto = CreateAlertDto {
        severity: alert.severity,
        identifier: alert.identifier,
        value: alert.value,
        note: alert.note,
        datasource_id: config.datasource_id.clone(),
    };

    let body = serde_json::to_vec(&dto).map_err(|_| Error::SerializationError)?;
    let url = format!("{}/alert", scripting_api.url);
    let cookie = format!("auth-token={}", scripting_api.token);

    let headers = [
        ("Content-Type", "application/json"),
        ("Cookie", cookie.as_str()),
    ];

    use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
    use embedded_svc::http::client::Client;

    let mut client = Client::wrap(
        EspHttpConnection::new(&Configuration::default()).map_err(Error::Esp)?,
    );

    let mut request = client
        .post(&url, &headers)
        .map_err(|_| Error::IoError)?;

    use embedded_io::Write;
    request.write_all(&body).map_err(|_| Error::IoError)?;
    request.flush().map_err(|_| Error::IoError)?;

    let response = request.submit().map_err(|e| {
        log::warn!("alert POST submit failed: {:?}", e);
        Error::IoError
    })?;

    let status = response.status();
    if status >= 400 {
        log::warn!("alert POST returned HTTP {}", status);
        return Err(Error::HttpStatus(status));
    }
    Ok(())
}
