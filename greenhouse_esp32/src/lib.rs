pub mod config;
pub mod device_builder;
pub mod device_service;
pub mod error;
pub mod handler;
pub mod hybrid_device;
pub mod input_device;
pub mod output_device;

pub use config::Config;
pub use device_builder::DeviceBuilder;

/// HTTP status code — mirrors the same constants as `axum::http::StatusCode`
/// so handler signatures need only drop `async` to be ported from the Linux version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(pub u16);

impl StatusCode {
    pub const OK: Self = StatusCode(200);
    pub const BAD_REQUEST: Self = StatusCode(400);
    pub const INTERNAL_SERVER_ERROR: Self = StatusCode(500);

    pub fn as_u16(self) -> u16 {
        self.0
    }
}
