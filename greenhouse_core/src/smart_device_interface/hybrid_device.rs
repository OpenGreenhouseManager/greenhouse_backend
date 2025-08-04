use axum::{
    Router,
    routing::{get, post},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    smart_device_dto::endpoints::{ACTIVATE, CONFIG, READ, STATUS, WRITE},
    smart_device_interface::handler::activate_device,
};

use super::{
    device_service::DeviceService,
    handler::{
        config_update_handler, get_config_handler, read_device_handler, status_device_handler,
        write_device_handler,
    },
};

pub fn init_hybrid_router<T>(device_service: DeviceService<T>) -> Router
where
    T: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    Router::new()
        .route(READ, get(read_device_handler))
        .route(WRITE, post(write_device_handler))
        .route(CONFIG, post(config_update_handler))
        .route(CONFIG, get(get_config_handler))
        .route(STATUS, get(status_device_handler))
        .route(ACTIVATE, post(activate_device))
        .with_state(device_service)
}
