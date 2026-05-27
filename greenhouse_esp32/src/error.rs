/// Result type alias for the ESP32 smart-device interface.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur in the ESP32 smart-device interface.
#[derive(Debug)]
pub enum Error {
    /// A configuration value in NVS could not be deserialised.
    IllFormattedConfig,
    /// No configuration has been written to NVS yet.
    MissingConfig,
    /// An alert was triggered but the device has not been activated with a
    /// scripting API, so the alert cannot be forwarded.
    ScriptingApiNotConfigured,
    /// An ESP-IDF system error (e.g. NVS or HTTP client failure).
    Esp(esp_idf_svc::sys::EspError),
    /// The scripting service returned a non-2xx HTTP status code.
    HttpStatus(u16),
    /// An I/O error occurred during an HTTP request.
    IoError,
    /// JSON serialisation or deserialisation failed.
    SerializationError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<esp_idf_svc::sys::EspError> for Error {
    fn from(e: esp_idf_svc::sys::EspError) -> Self {
        Error::Esp(e)
    }
}
