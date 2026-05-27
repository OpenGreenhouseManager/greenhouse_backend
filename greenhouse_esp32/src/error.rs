pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IllFormattedConfig,
    MissingConfig,
    ScriptingApiNotConfigured,
    Esp(esp_idf_svc::sys::EspError),
    HttpStatus(u16),
    IoError,
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
