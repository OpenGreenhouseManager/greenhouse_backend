mod error;
pub(crate) mod middleware;
pub(crate) mod router;
pub(crate) mod service;

pub(crate) use self::error::{Error, Result};
pub(crate) const AUTH_TOKEN: &str = "auth-token";
