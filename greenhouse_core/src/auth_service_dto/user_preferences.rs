use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserPreferencesRequestDto {
    pub dashboard_preferences: serde_json::Value,
    pub alert_preferences: serde_json::Value,
}

#[cfg(feature = "error_handling")]
impl axum::response::IntoResponse for UserPreferencesRequestDto {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserPreferencesResponseDto {
    pub dashboard_preferences: serde_json::Value,
    pub alert_preferences: serde_json::Value,
}

#[cfg(feature = "error_handling")]
impl axum::response::IntoResponse for UserPreferencesResponseDto {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetPreferencesRequestDto {
    pub token: String,
    pub preferences: UserPreferencesRequestDto,
}

#[cfg(feature = "error_handling")]
impl axum::response::IntoResponse for SetPreferencesRequestDto {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}
