use axum::{
    Router,
    routing::{get, post},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    smart_device_dto::endpoints::{ACTIVATE, CONFIG, READ, STATUS},
    smart_device_interface::handler::activate_device,
};

use super::{
    device_builder::DeviceBuilder,
    handler::{
        config_update_handler, get_config_handler, read_device_handler, status_device_handler,
    },
};

/// Builds an Axum [`Router`] for an output (sensor) device.
///
/// Registers the following routes on the returned router:
///
/// | Method | Path | Handler |
/// |--------|------|---------|
/// | `GET` | `/read` | `read_device_handler` — returns the current sensor reading |
/// | `GET` | `/status` | `status_device_handler` — returns online/panic status |
/// | `GET` | `/config` | `get_config_handler` — returns current configuration |
/// | `POST` | `/config` | `config_update_handler` — updates additional config |
/// | `POST` | `/activate` | `activate_device` — registers scripting API credentials |
///
/// Pass the router to `axum::serve` to start the device HTTP server.
pub fn init_output_router<T>(device_service: DeviceBuilder<T>) -> Router
where
    T: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    Router::new()
        .route(READ, get(read_device_handler))
        .route(CONFIG, post(config_update_handler))
        .route(CONFIG, get(get_config_handler))
        .route(ACTIVATE, post(activate_device))
        .route(STATUS, get(status_device_handler))
        .with_state(device_service)
}
