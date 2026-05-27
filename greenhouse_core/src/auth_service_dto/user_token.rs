use serde::{Deserialize, Serialize};

/// Decoded JWT claims stored in the auth-token cookie.
///
/// These fields follow the standard JWT claim names registered by IANA.
///
/// # Example
///
/// ```
/// # use greenhouse_core::auth_service_dto::user_token::UserToken;
/// let token = UserToken {
///     iat: 1_700_000_000,
///     exp: 1_700_086_400,
///     user_name: "alice".to_string(),
///     role: "ADMIN".to_string(),
/// };
/// assert_eq!(token.role, "ADMIN");
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct UserToken {
    /// Issued-at timestamp (Unix seconds). Wire-format name: `iat`.
    pub iat: i64,
    /// Expiration timestamp (Unix seconds). Wire-format name: `exp`.
    pub exp: i64,
    /// Username of the token holder.
    pub user_name: String,
    /// Role of the token holder (e.g. `"ADMIN"`, `"GUEST"`).
    pub role: String,
}
