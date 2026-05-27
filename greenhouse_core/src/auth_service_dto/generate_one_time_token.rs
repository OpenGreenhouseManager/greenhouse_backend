use serde::{Deserialize, Serialize};

/// Request body for generating a single-use invitation token.
///
/// An admin submits this to create a token that a new user can supply during
/// guest registration.
#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateOneTimeTokenRequestDto {
    /// Username of the admin requesting the token (for audit purposes).
    pub username: String,
}

/// Response containing the generated single-use invitation token.
#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateOneTimeTokenResponseDto {
    /// The single-use token string. Provide this to the user who will register.
    pub token: String,
}
