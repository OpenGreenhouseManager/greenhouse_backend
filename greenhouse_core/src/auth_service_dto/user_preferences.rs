use axum::{
    Json,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserPreferencesRequestDto {
    pub dashboard_preferences: serde_json::Value,
    pub alert_preferences: serde_json::Value,
}

impl IntoResponse for UserPreferencesRequestDto {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserPreferencesResponseDto {
    pub dashboard_preferences: serde_json::Value,
    pub alert_preferences: serde_json::Value,
}

impl IntoResponse for UserPreferencesResponseDto {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
