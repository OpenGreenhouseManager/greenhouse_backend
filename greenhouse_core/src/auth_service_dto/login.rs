use serde::{Deserialize, Serialize};

/// Credentials submitted to the login endpoint.
///
/// # Example
///
/// ```
/// # use greenhouse_core::auth_service_dto::login::LoginRequestDto;
/// let req = LoginRequestDto {
///     username: "alice".to_string(),
///     password: "s3cr3t".to_string(),
/// };
/// assert_eq!(req.username, "alice");
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRequestDto {
    /// Account username.
    pub username: String,
    /// Account password (plain text; the service hashes it with bcrypt).
    pub password: String,
}

/// Successful login response containing a JWT.
///
/// # Example
///
/// ```
/// # use greenhouse_core::auth_service_dto::login::LoginResponseDto;
/// let resp = LoginResponseDto {
///     token: "eyJ...".to_string(),
///     token_type: "Bearer".to_string(),
/// };
/// assert_eq!(resp.token_type, "Bearer");
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponseDto {
    /// The issued JWT string.
    pub token: String,
    /// Token scheme, typically `"Bearer"`.
    pub token_type: String,
}
