pub(crate) mod middleware;
pub(crate) mod service;

pub(crate) use crate::helper::error::{Error, Result};
pub(crate) const AUTH_TOKEN: &str = "auth-token";
