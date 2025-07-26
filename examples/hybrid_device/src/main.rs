use std::sync::Arc;

use axum::{Json, http::StatusCode};
use greenhouse_core::{
    smart_device_dto::{
        config::ConfigRequestDto,
        status::{DeviceStatusDto, DeviceStatusResponseDto},
    },
    smart_device_interface::{
        config::{read_config_file, update_config_file, Config, Mode, Type},
        device_service::DeviceService,
        hybrid_device::init_hybrid_router,
    },
};
use serde_derive::{Deserialize, Serialize};

static mut SAVED_NUMBER: i32 = 20;
const DATASOURCE_ID: &str = "7a224a14-6e07-45a3-91da-b7584a5731c1";

#[derive(Serialize, Deserialize, Clone, Default)]
struct ExampleDeviceConfig {
    pub min: i32,
    pub max: i32,
}

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() {
    let config = match read_config_file() {
        Ok(config) => config,
        Err(_) => {
            let default_config = Config {
                mode: Mode::InputOutput,
                port: 6001,
                input_type: Some(Type::Number),
                output_type: Some(Type::Number),
                additional_config: ExampleDeviceConfig { min: 0, max: 100 },
            };
            // check if config file exists
            if !std::path::Path::new("./config/config.json").exists() {
                // create config directory
                std::fs::create_dir_all("./config").unwrap();
                // create config file
                std::fs::write("./config/config.json", "{}").unwrap();
            }
            update_config_file(&default_config).unwrap();
            default_config
        }
    };

    let device_service = DeviceService::new_hybrid_device(
        read_handler,
        write_handler,
        status_handler,
        config_interceptor_handler,
    )
    .unwrap();
    let router = init_hybrid_router(device_service);

    let url = format!("0.0.0.0:{}", config.port);

    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}

fn read_handler(_: Arc<Config<ExampleDeviceConfig>>) -> String {
    // Implement your read handler here
    Json(unsafe { SAVED_NUMBER }).to_string()
}

fn write_handler(json: String, config: Arc<Config<ExampleDeviceConfig>>) -> StatusCode {
    // Implement your write handler here
    let number: i32 = json.parse().unwrap();
    unsafe { SAVED_NUMBER = number };
    if config.additional_config.min > number || config.additional_config.max < number {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

fn status_handler(_: Arc<Config<ExampleDeviceConfig>>) -> DeviceStatusResponseDto {
    // Implement your status handler here
    DeviceStatusResponseDto {
        status: DeviceStatusDto::Online,
        datasource_id: From::from(DATASOURCE_ID),
    }
}

fn config_interceptor_handler(
    config: ConfigRequestDto<ExampleDeviceConfig>,
    old_config: Arc<Config<ExampleDeviceConfig>>,
) -> Config<ExampleDeviceConfig> {
    // Implement your config interceptor handler here
    Config {
        mode: old_config.mode.clone(),
        port: old_config.port,
        input_type: old_config.input_type,
        output_type: old_config.output_type,
        additional_config: {
            ExampleDeviceConfig {
                min: config.additional_config.min,
                max: config.additional_config.max,
            }
        },
    }
}
