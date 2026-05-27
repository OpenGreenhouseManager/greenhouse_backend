//! Minimal example of creating a smart-device HTTP server with [`DeviceBuilder`].
//!
//! Shows the complete boilerplate for an output (sensor) device that returns a
//! static temperature reading. In production, the handlers would read from real
//! hardware.
//!
//! **Requires** a config file at `./config/config.json` (or the path set in
//! `CONFIG_PATH`) before running. Create one manually:
//!
//! ```json
//! {"port": 8080, "datasource_id": "my-uuid", "additional_config": {}, "scripting_api": null}
//! ```
//!
//! Run with:
//! ```bash
//! cargo run --example device_builder_minimal --features smart_device_interface
//! ```

use greenhouse_core::smart_device_dto::{
    Measurement, Type,
    config::TypeOption,
    status::{DeviceStatusDto, DeviceStatusResponseDto},
    config::ConfigRequestDto,
};
use greenhouse_core::smart_device_interface::{
    config::Config,
    device_builder::DeviceBuilder,
    output_device::init_output_router,
};
use std::sync::Arc;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
struct MyConfig {
    /// Configurable sensor label.
    pub label: String,
}

/// Returns the current sensor reading.
async fn read_handler(cfg: Arc<Config<MyConfig>>) -> Type {
    Type::Measurement(Measurement {
        value: 22.5,
        unit: "°C".to_string(),
    })
}

/// Returns the device status.
async fn status_handler(cfg: Arc<Config<MyConfig>>) -> DeviceStatusResponseDto {
    DeviceStatusResponseDto {
        status: DeviceStatusDto::Online,
        datasource_id: cfg.datasource_id.clone(),
    }
}

/// Config interceptor — applies and returns the new config unchanged.
async fn config_interceptor(
    req: ConfigRequestDto<MyConfig>,
    current: Arc<Config<MyConfig>>,
) -> Config<MyConfig> {
    Config {
        additional_config: req.additional_config,
        ..(*current).clone()
    }
}

#[tokio::main]
async fn main() {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "./config/config.json".to_string());

    let builder = DeviceBuilder::new_output_device_with_config_path(
        read_handler,
        status_handler,
        config_interceptor,
        &config_path,
        TypeOption::Measurement,
    )
    .expect("failed to build device — does config.json exist?");

    let router = init_output_router(builder);

    // Determine listen address from the config (already loaded into the builder).
    println!("Starting device server — send GET /read to get a reading.");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind");

    axum::serve(listener, router)
        .await
        .expect("server error");
}
