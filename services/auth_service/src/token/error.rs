use serde::Serialize;

pub(crate) type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, PartialEq)]
pub(crate) enum Error {
    InvalidTime,
    JwtEncode,
    JwtDecode,
    RegisterToken,
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
