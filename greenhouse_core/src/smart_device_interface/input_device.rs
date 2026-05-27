use axum::{
    Router,
    routing::{get, post},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    smart_device_dto::endpoints::{ACTIVATE, CONFIG, STATUS, WRITE},
    smart_device_interface::handler::activate_device,
};

use super::{
    device_builder::DeviceBuilder,
    handler::{
        config_update_handler, get_config_handler, status_device_handler, write_device_handler,
    },
};

/// Builds an Axum [`Router`] for an input (actuator) device.
///
/// Registers the following routes on the returned router:
///
/// | Method | Path | Handler |
/// |--------|------|---------|
/// | `POST` | `/write` | `write_device_handler` — forwards a value to the actuator |
/// | `GET` | `/status` | `status_device_handler` — returns online/panic status |
/// | `GET` | `/config` | `get_config_handler` — returns current configuration |
/// | `POST` | `/config` | `config_update_handler` — updates additional config |
/// | `POST` | `/activate` | `activate_device` — registers scripting API credentials |
///
/// Pass the router to `axum::serve` to start the device HTTP server.
pub fn init_input_router<T>(device_service: DeviceBuilder<T>) -> Router
where
    T: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    Router::new()
        .route(WRITE, post(write_device_handler))
        .route(CONFIG, post(config_update_handler))
        .route(CONFIG, get(get_config_handler))
        .route(STATUS, get(status_device_handler))
        .route(ACTIVATE, post(activate_device))
        .with_state(device_service)
}
