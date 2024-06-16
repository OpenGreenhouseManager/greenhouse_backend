use axum::{
    routing::{get, post},
    Router,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::smart_device_dto::endpoints::{CONFIG, STATUS, WRITE};

use super::{
    device_service::DeviceService,
    handler::{
        config_update_handler, get_config_handler, status_device_handler, write_device_handler,
    },
};

pub fn init_input_router<T>(device_service: DeviceService<T>) -> Router
where
    T: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    Router::new()
        .route(WRITE, post(write_device_handler))
        .route(CONFIG, post(config_update_handler))
        .route(CONFIG, get(get_config_handler))
        .route(STATUS, get(status_device_handler))
        .with_state(device_service)
}
