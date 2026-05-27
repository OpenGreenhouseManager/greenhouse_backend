//! Demonstrates constructing and serialising the most common DTO types.
//!
//! Run with:
//! ```bash
//! cargo run --example dto_construction --features auth_service_dto,device_service_dto,data_storage_service_dto
//! ```

use greenhouse_core::auth_service_dto::login::{LoginRequestDto, LoginResponseDto};
use greenhouse_core::data_storage_service_dto::alert_dto::alert::{AlertDto, AlertsDto, Severity};
use greenhouse_core::device_service_dto::get_device::{DeviceResponseDto, DevicesResponseDto};

fn main() {
    // --- Auth DTOs ---
    let login_req = LoginRequestDto {
        username: "alice".to_string(),
        password: "s3cr3t".to_string(),
    };
    println!("Login request: {}", serde_json::to_string_pretty(&login_req).unwrap());

    let login_resp = LoginResponseDto {
        token: "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...".to_string(),
        token_type: "Bearer".to_string(),
    };
    println!("Login response: {}", serde_json::to_string_pretty(&login_resp).unwrap());

    // --- Alert DTOs ---
    let alert = AlertDto {
        id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        severity: Severity::Warning,
        identifier: "temperature_high".to_string(),
        value: "35.2".to_string(),
        note: Some("Exceeded 35°C threshold".to_string()),
        created_at: "2024-06-01T14:30:00Z".to_string(),
        datasource_id: "device-uuid-here".to_string(),
    };

    let alerts: AlertsDto = vec![alert].into();
    println!("Alerts: {}", serde_json::to_string_pretty(&alerts).unwrap());

    // --- Device DTOs ---
    let device = DeviceResponseDto {
        id: "device-uuid-here".to_string(),
        name: "Greenhouse Temp Sensor".to_string(),
        address: "http://192.168.1.10:8080".to_string(),
        description: "Main greenhouse temperature and humidity sensor".to_string(),
        canscript: true,
        scraping: true,
    };

    let devices: DevicesResponseDto = vec![device].into();
    println!("Devices: {}", serde_json::to_string_pretty(&devices).unwrap());
}
