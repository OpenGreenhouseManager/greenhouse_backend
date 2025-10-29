use std::sync::Arc;

use greenhouse_core::smart_device_interface::SmartDeviceOpResult;
use greenhouse_core::{
    smart_device_dto::{
        Type,
        config::ConfigRequestDto,
        status::{DeviceStatusDto, DeviceStatusResponseDto},
    },
    smart_device_interface::{
        config::{
            Config, Mode, TypeOptionDto, read_config_file_with_path, update_config_file_with_path,
        },
        device_builder::DeviceBuilder,
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
    // Get config path from command line arguments, default to "./config/config.json"
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./config/input_output_int_saver/config.json".to_string());

    let config = match read_config_file_with_path(&config_path) {
        Ok(config) => config,
        Err(_) => {
            let default_config = Config {
                mode: Mode::InputOutput,
                port: 6001,
                datasource_id: DATASOURCE_ID.to_string(),
                input_type: Some(TypeOptionDto::Number),
                output_type: Some(TypeOptionDto::Number),
                additional_config: ExampleDeviceConfig { min: 0, max: 100 },
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

    let device_service = DeviceBuilder::new_hybrid_device_with_config_path(
        read_handler,
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

async fn read_handler(_: Arc<Config<ExampleDeviceConfig>>) -> SmartDeviceOpResult<Type> {
    SmartDeviceOpResult::Result(Type::Number(unsafe { SAVED_NUMBER } as f64))
}

async fn write_handler(
    data: Type,
    config: Arc<Config<ExampleDeviceConfig>>,
) -> SmartDeviceOpResult<()> {
    let number = match data {
        Type::Number(number) => number,
        _ => {
            return SmartDeviceOpResult::Error {
                status_code: 400,
                message: "expected number".to_string(),
            }
        }
    };
    unsafe { SAVED_NUMBER = number as i32 };
    if config.additional_config.min > number as i32 || config.additional_config.max < number as i32
    {
        return SmartDeviceOpResult::Error {
            status_code: 500,
            message: "value out of range".to_string(),
        };
    }
    SmartDeviceOpResult::Result(())
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
