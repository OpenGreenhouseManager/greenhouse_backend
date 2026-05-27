use serde::{Deserialize, Serialize};

/// Admin account registration request.
///
/// Used to create the initial administrator account. No invitation token is
/// required, but the endpoint is typically restricted to the first call or
/// guarded by server-side logic.
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterAdminRequestDto {
    /// Desired admin username.
    pub username: String,
    /// Desired admin password (plain text; the service hashes it with bcrypt).
    pub password: String,
}

/// Successful admin registration response containing a JWT.
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterAdminResponseDto {
    /// The issued JWT string for the admin account.
    pub token: String,
    /// Token scheme, typically `"Bearer"`.
    pub token_type: String,
}
