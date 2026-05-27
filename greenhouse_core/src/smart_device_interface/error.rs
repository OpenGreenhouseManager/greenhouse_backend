/// Result type alias for the smart device interface.
///
/// All fallible operations in this module return `Result<T>` which expands to
/// `core::result::Result<T, Error>`.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur in the smart device interface.
#[derive(Debug)]
pub enum Error {
    /// The configuration file exists but could not be deserialised.
    IllFormattedConfig,
    /// No configuration file was found at the expected path.
    MissingConfig,
    /// An alert was triggered but the device has not been activated with a
    /// scripting API, so the alert cannot be forwarded.
    ScriptingApiNotConfigured,
    /// An HTTP request to the scripting service failed.
    Request(reqwest::Error),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
