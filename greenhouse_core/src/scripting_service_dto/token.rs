use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

/// A scripting service authentication token.
///
/// Devices receive this token after activating via the scripting service and
/// include it in subsequent API calls (e.g. when posting alerts).
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct TokenDto {
    /// The bearer token string.
    pub token: String,
}
