pub mod config;
pub mod device_builder;
pub mod device_service;
mod error;
#[cfg(feature = "smart_device_interface_axum")]
mod handler;
#[cfg(feature = "smart_device_interface_axum")]
pub mod hybrid_device;
#[cfg(feature = "smart_device_interface_axum")]
pub mod input_device;
pub mod op_result;
#[cfg(feature = "smart_device_interface_axum")]
pub mod output_device;

pub use self::error::{Error, Result};
pub use self::op_result::SmartDeviceOpResult;
