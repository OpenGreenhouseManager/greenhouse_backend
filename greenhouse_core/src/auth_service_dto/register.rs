use serde::{Deserialize, Serialize};

/// Guest registration request — requires a valid one-time invitation token.
///
/// An admin must first call the `generate_one_time_token` endpoint to obtain a
/// single-use token that authorises this registration.
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequestDto {
    /// Desired account username.
    pub username: String,
    /// Desired account password (plain text; the service hashes it with bcrypt).
    pub password: String,
    /// Single-use token issued by an admin via [`generate_one_time_token`](super::generate_one_time_token).
    pub one_time_token: String,
}

/// Successful guest registration response containing a JWT.
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponseDto {
    /// The issued JWT string for the newly created account.
    pub token: String,
    /// Token scheme, typically `"Bearer"`.
    pub token_type: String,
}
