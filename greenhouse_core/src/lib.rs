#[cfg(feature = "auth_service_dto")]
pub mod auth_service_dto;
#[cfg(feature = "data_storage_service_dto")]
pub mod data_storage_service_dto;
#[cfg(feature = "device_service_dto")]
pub mod device_service_dto;
#[cfg(feature = "smart_device_dto")]
pub mod smart_device_dto;
#[cfg(feature = "smart_device_interface")]
pub mod smart_device_interface;

// Error handling module for standardized API error responses
pub mod error;
