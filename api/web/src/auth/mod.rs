mod error;
pub mod middleware;
pub mod router;
pub mod service;

pub use self::error::{Error, Result};
pub const AUTH_TOKEN: &str = "auth-token";
