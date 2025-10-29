use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SmartDeviceOpResult<T> {
    Result(T),
    Error { status_code: i32, message: String },
}
