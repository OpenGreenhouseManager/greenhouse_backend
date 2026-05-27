use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

/// User preference data included in requests that update preferences.
///
/// Both fields hold arbitrary JSON, giving the frontend flexibility to evolve
/// preference schemas without backend changes.
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct UserPreferencesRequestDto {
    /// Dashboard layout and widget configuration as a JSON value.
    pub dashboard_preferences: serde_json::Value,
    /// Alert notification and display settings as a JSON value.
    pub alert_preferences: serde_json::Value,
}

/// User preference data returned by the preferences endpoint.
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct UserPreferencesResponseDto {
    /// Dashboard layout and widget configuration as a JSON value.
    pub dashboard_preferences: serde_json::Value,
    /// Alert notification and display settings as a JSON value.
    pub alert_preferences: serde_json::Value,
}

/// Request body for the set-preferences endpoint, combining a JWT and the new preferences.
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
pub struct SetPreferencesRequestDto {
    /// JWT string identifying the user whose preferences are being updated.
    pub token: String,
    /// The new preference values to store.
    pub preferences: UserPreferencesRequestDto,
}
