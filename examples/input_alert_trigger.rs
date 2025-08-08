use std::sync::Arc;

use axum::http::StatusCode;
use greenhouse_core::{
    data_storage_service_dto::alert_dto::alert::Severity,
    smart_device_dto::{
        config::ConfigRequestDto,
        status::{DeviceStatusDto, DeviceStatusResponseDto},
    },
    smart_device_interface::{
        config::{Config, Mode, Type, read_config_file_with_path, update_config_file_with_path},
        device_builder::DeviceBuilder,
        device_service::{AlertCreation, trigger_alert},
        hybrid_device::init_hybrid_router,
    },
};
use serde_derive::{Deserialize, Serialize};

const DATASOURCE_ID: &str = "7a224a14-6e07-45a3-91da-b7584a5731c1";
const LOW_ALERT_IDENTIFIER: &str = "low_alert";
const HIGH_ALERT_IDENTIFIER: &str = "high_alert";

#[derive(Serialize, Deserialize, Clone, Default)]
struct ExampleDeviceConfig {
    pub min: i32,
    pub max: i32,
}

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() {
    // Get config path from command line arguments, default to "./config/config.json"
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./config/config.json".to_string());

    let config = match read_config_file_with_path(&config_path) {
        Ok(config) => config,
        Err(_) => {
            let default_config = Config {
                mode: Mode::InputOutput,
                port: 6002,
                datasource_id: DATASOURCE_ID.to_string(),
                input_type: Some(Type::Number),
                output_type: None,
                additional_config: ExampleDeviceConfig { min: 0, max: 10 },
                scripting_api: None,
            };

            // Create config directory if it doesn't exist
            if let Some(parent) = std::path::Path::new(&config_path).parent()
                && !parent.exists()
            {
                std::fs::create_dir_all(parent).unwrap();
            }

            // Create empty config file if it doesn't exist
            if !std::path::Path::new(&config_path).exists() {
                std::fs::write(&config_path, "{}").unwrap();
            }

            update_config_file_with_path(&default_config, &config_path).unwrap();
            default_config
        }
    };

    let device_service = DeviceBuilder::new_input_device_with_config_path(
        write_handler,
        status_handler,
        config_interceptor_handler,
        &config_path,
    )
    .unwrap();
    let router = init_hybrid_router(device_service);

    let url = format!("0.0.0.0:{}", config.port);

    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    println!("using config file: {config_path}");
    axum::serve(listener, router).await.unwrap();
}

async fn write_handler(json: String, config: Arc<Config<ExampleDeviceConfig>>) -> StatusCode {
    let number: i32 = json.parse().unwrap();
    if number > config.additional_config.max {
        trigger_alert(
            config.clone(),
            AlertCreation {
                severity: Severity::Error,
                identifier: HIGH_ALERT_IDENTIFIER.to_string(),
                value: Some(number.to_string()),
                note: None,
            },
        )
        .await
        .unwrap();
    }
    if number < config.additional_config.min {
        trigger_alert(
            config.clone(),
            AlertCreation {
                severity: Severity::Error,
                identifier: LOW_ALERT_IDENTIFIER.to_string(),
                value: Some(number.to_string()),
                note: None,
            },
        )
        .await
        .unwrap();
    }
    StatusCode::OK
}

async fn status_handler(config: Arc<Config<ExampleDeviceConfig>>) -> DeviceStatusResponseDto {
    DeviceStatusResponseDto {
        status: DeviceStatusDto::Online,
        datasource_id: config.datasource_id.clone(),
    }
}

async fn config_interceptor_handler(
    config: ConfigRequestDto<ExampleDeviceConfig>,
    old_config: Arc<Config<ExampleDeviceConfig>>,
) -> Config<ExampleDeviceConfig> {
    Config {
        mode: old_config.mode.clone(),
        port: old_config.port,
        datasource_id: old_config.datasource_id.clone(),
        input_type: old_config.input_type,
        output_type: old_config.output_type,
        additional_config: {
            ExampleDeviceConfig {
                min: config.additional_config.min,
                max: config.additional_config.max,
            }
        },
        scripting_api: old_config.scripting_api.clone(),
    }
}
