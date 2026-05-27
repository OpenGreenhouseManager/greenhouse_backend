use serde::{Deserialize, Serialize};

/// Token validation request sent to the `check_token` endpoint.
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenRequestDto {
    /// The raw JWT string to validate.
    pub token: String,
    /// Token scheme (e.g. `"Bearer"`).
    pub token_type: String,
}

/// Response from the `check_token` endpoint describing the token holder's role.
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponseDto {
    /// The role embedded in the validated token (e.g. `"ADMIN"`, `"GUEST"`).
    pub role: String,
}
