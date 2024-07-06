use std::sync::Arc;

use axum::{http::StatusCode, Json};
use greenhouse_core::{
    smart_device_dto::{config::ConfigRequestDto, status::DeviceStatusResponseDto},
    smart_device_interface::{
        config::{Config, Mode, Type},
        device_service::DeviceService,
        hybrid_device::init_hybrid_router,
    },
};
use serde_derive::{Deserialize, Serialize};

static mut SAVED_NUMBER: i32 = 20;

#[derive(Serialize, Deserialize, Clone, Default)]
struct ExampleDeviceConfig {
    pub min: i32,
    pub max: i32,
}

#[tokio::main]
async fn main() {
    let device_service = DeviceService::new_hybrid_device(
        read_handler,
        write_handler,
        status_handler,
        config_interceptor_handler,
    )
    .unwrap();
    let router = init_hybrid_router(device_service);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
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
    if config.additinal_config.min > number || config.additinal_config.max < number {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

fn status_handler(_: Arc<Config<ExampleDeviceConfig>>) -> DeviceStatusResponseDto {
    // Implement your status handler here
    DeviceStatusResponseDto::Online
}

fn config_interceptor_handler(
    config: ConfigRequestDto<ExampleDeviceConfig>,
) -> Config<ExampleDeviceConfig> {
    // Implement your config interceptor handler here
    Config {
        mode: Mode::InputOutput,
        input_type: Some(Type::Number),
        output_type: Some(Type::String),
        additinal_config: {
            ExampleDeviceConfig {
                min: config.additinal_config.min,
                max: config.additinal_config.max,
            }
        },
    }
}
