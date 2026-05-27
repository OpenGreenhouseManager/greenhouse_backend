use serde::{Deserialize, Serialize};

/// Request body sent to the device `/activate` endpoint.
///
/// The device service calls this endpoint to register a smart device with the
/// scripting service, supplying the URL and authentication token that the device
/// will use for subsequent API calls (e.g. posting alerts).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActivateRequestDto {
    /// Base URL of the scripting service (e.g. `"http://scripting:3003"`).
    pub url: String,
    /// Bearer token for authenticating with the scripting service.
    pub token: String,
}
