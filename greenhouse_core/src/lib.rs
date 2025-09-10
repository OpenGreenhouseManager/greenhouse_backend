#[cfg(feature = "auth_service_dto")]
pub mod auth_service_dto;
#[cfg(feature = "data_storage_service_dto")]
pub mod data_storage_service_dto;
#[cfg(feature = "device_service_dto")]
pub mod device_service_dto;
#[cfg(feature = "scripting_service_dto")]
pub mod scripting_service_dto;
#[cfg(feature = "smart_device_dto")]
pub mod smart_device_dto;
#[cfg(feature = "smart_device_interface")]
pub mod smart_device_interface;

// HTTP error mapping system - enabled when axum is available
#[cfg(feature = "error_handling")]
pub mod http_error;
