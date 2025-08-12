use std::sync::Arc;

use greenhouse_core::{
    data_storage_service_dto::alert_dto::alert::Severity,
    smart_device_dto::{
        config::ConfigRequestDto,
        status::{DeviceStatusDto, DeviceStatusResponseDto},
    },
    smart_device_interface::{
        Error,
        config::{Config, Mode, Type, read_config_file_with_path, update_config_file_with_path},
        device_builder::DeviceBuilder,
        device_service::{AlertCreation, trigger_alert},
        hybrid_device::init_hybrid_router,
    },
};
use serde_derive::{Deserialize, Serialize};

const DATASOURCE_ID: &str = "7a224a14-6e07-45a3-91da-b7584a5731c1";
const PERIODIC_ALERT_IDENTIFIER_LIST: [&str; 5] = [
    "periodic_alert_1",
    "periodic_alert_2",
    "periodic_alert_3",
    "periodic_alert_4",
    "periodic_alert_5",
];

#[derive(Serialize, Deserialize, Clone, Default)]
struct ExampleDeviceConfig {
    pub interval: u64,
    pub random_jitter: u64,
}

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() {
    // Get config path from command line arguments, default to "./config/config.json"
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./config/periodic_alert/config.json".to_string());

    let config = match read_config_file_with_path(&config_path) {
        Ok(config) => config,
        Err(_) => {
            let default_config = Config {
                mode: Mode::InputOutput,
                port: 6003,
                datasource_id: DATASOURCE_ID.to_string(),
                input_type: Some(Type::Number),
                output_type: None,
                additional_config: ExampleDeviceConfig {
                    interval: 10,
                    random_jitter: 5,
                },
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

    let device_service = DeviceBuilder::new_output_device_with_config_path(
        read_handler,
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

    // Run periodic alerts in a background task, but avoid moving non-Send types into the task.
    tokio::spawn({
        async move {
            start_periodic_alerts(config, &config_path).await;
        }
    });

    axum::serve(listener, router).await.unwrap();
}

async fn read_handler(_: Arc<Config<ExampleDeviceConfig>>) -> String {
    "asd".to_string()
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
                interval: config.additional_config.interval,
                random_jitter: config.additional_config.random_jitter,
            }
        },
        scripting_api: old_config.scripting_api.clone(),
    }
}

async fn start_periodic_alerts(config: Config<ExampleDeviceConfig>, config_path: &str) {
    let mut config = config;
    let interval = config.additional_config.interval;
    let mut last_alert_time = std::time::Instant::now();

    loop {
        let current_time = std::time::Instant::now();
        if current_time.duration_since(last_alert_time) >= std::time::Duration::from_secs(interval)
        {
            let jitter = 1;
            let next_alert_time = current_time + std::time::Duration::from_secs(interval + jitter);
            last_alert_time = next_alert_time;
            let random_index = 1;
            let random_severity = 1;
            let res = trigger_alert(
                Arc::new(config.clone()),
                AlertCreation {
                    identifier: PERIODIC_ALERT_IDENTIFIER_LIST[random_index].to_string(),
                    severity: match random_severity {
                        0 => Severity::Info,
                        1 => Severity::Warning,
                        2 => Severity::Error,
                        3 => Severity::Fatal,
                        _ => panic!("Invalid severity"),
                    },
                    value: None,
                    note: None,
                },
            )
            .await;
            match res {
                Ok(_) => println!("Alert triggered successfully"),
                Err(e) => match e {
                    Error::ScriptingApiNotConfigured => {
                        println!("Scripting API not configured, reloading config");
                        config = read_config_file_with_path(config_path).unwrap();
                    }
                    _ => println!("Error triggering alert: {e}"),
                },
            }
        }
    }
}
