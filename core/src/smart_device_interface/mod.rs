use axum::{ routing::{ get, post }, Router, Json, http::StatusCode, extract::State };
use serde::{ Deserialize, Serialize };
use std::sync::Arc;

#[derive(Clone)]
pub struct DeviceService {
    read_handler: Arc<dyn (Fn() -> String) + Send + Sync>,
    write_handler: Arc<dyn Fn(String) + Send + Sync>,
}

#[derive(Serialize, Deserialize)]
struct WritePayload {
    data: String,
}

impl DeviceService {
    pub fn new(
        read_handler: impl (Fn() -> String) + Send + Sync + 'static,
        write_handler: impl Fn(String) + Send + Sync + 'static
    ) -> Self {
        DeviceService {
            read_handler: Arc::new(read_handler),
            write_handler: Arc::new(write_handler),
        }
    }
}

pub fn create_router(device_service: DeviceService) -> Router {
    Router::new()
        .route("/read_device", get(read_device_handler))
        .route("/write_device", post(write_device_handler))
        .with_state(device_service)
}

async fn read_device_handler(State(device_service): State<DeviceService>) -> Json<String> {
    let result = (device_service.read_handler)();
    Json(result)
}

async fn write_device_handler(
    State(device_service): State<DeviceService>,
    Json(payload): Json<WritePayload>
) -> StatusCode {
    (device_service.write_handler)(payload.data);
    StatusCode::OK
}

async fn main() {
    let read_handler = || {
        // Your custom implementation here
        "Read data from device".to_string()
    };

    let write_handler = |data: String| {
        // Your custom implementation here
        println!("Write data to device: {}", data);
    };

    let device_service = DeviceService::new(read, write_handler);
    let app = create_router(device_service);
}

fn read() -> String {
    "hi".to_string()
}
