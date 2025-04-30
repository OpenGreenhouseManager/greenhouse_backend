use axum::{
    Router,
    routing::{get, post},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::smart_device_dto::endpoints::{CONFIG, READ, STATUS};

use super::{
    device_service::DeviceService,
    handler::{
        config_update_handler, get_config_handler, read_device_handler, status_device_handler,
    },
};

pub fn init_output_router<T>(device_service: DeviceService<T>) -> Router
where
    T: Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    Router::new()
        .route(READ, get(read_device_handler))
        .route(CONFIG, post(config_update_handler))
        .route(CONFIG, get(get_config_handler))
        .route(STATUS, get(status_device_handler))
        .with_state(device_service)
}
