use greenhouse_macro::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, IntoResponse)]
pub struct UserPreferencesRequestDto {
    pub dashboard_preferences: serde_json::Value,
    pub alert_preferences: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, IntoResponse)]
pub struct UserPreferencesResponseDto {
    pub dashboard_preferences: serde_json::Value,
    pub alert_preferences: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, IntoResponse)]
pub struct SetPreferencesRequestDto {
    pub token: String,
    pub preferences: UserPreferencesRequestDto,
}
