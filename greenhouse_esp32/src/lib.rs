//! ESP32 implementation of the OpenGreenhouseManager smart-device interface.
//!
//! This crate mirrors the [`greenhouse_core::smart_device_interface`] API for ESP32
//! targets using [`esp-idf-svc`]. The key differences from the Linux/Axum version are:
//!
//! - **Synchronous handlers** — handler functions are plain `fn`, not `async fn`.
//! - **NVS persistence** — configuration is stored in ESP32 Non-Volatile Storage
//!   (NVS) via `esp-idf-svc` rather than a JSON file on disk.
//! - **`run()` instead of router init** — call [`DeviceBuilder::run`] at the end of
//!   `main`; it starts the ESP-IDF HTTP server and loops forever.
//! - **Custom [`StatusCode`]** — a thin wrapper around `u16` that mirrors
//!   `axum::http::StatusCode` so handler function signatures stay compatible.
//!
//! # Build requirements
//!
//! Targets `xtensa-esp32s3-espidf` and requires the ESP-IDF toolchain. See
//! [esp-idf-svc](https://github.com/esp-rs/esp-idf-svc) for installation
//! instructions. This crate is not published to crates.io (`publish = false`).
//!
//! # Example
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use greenhouse_esp32::{DeviceBuilder, StatusCode};
//! # use greenhouse_esp32::config::Config;
//! # use greenhouse_core::smart_device_dto::{
//! #     Type, config::{TypeOption, ConfigRequestDto},
//! #     status::{DeviceStatusDto, DeviceStatusResponseDto},
//! # };
//! # #[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
//! # struct MyCfg;
//! fn read(cfg: Arc<Config<MyCfg>>) -> Type {
//!     Type::Number(22.5)
//! }
//! fn write(value: Type, cfg: Arc<Config<MyCfg>>) -> StatusCode {
//!     StatusCode::OK
//! }
//! fn status(cfg: Arc<Config<MyCfg>>) -> DeviceStatusResponseDto {
//!     DeviceStatusResponseDto {
//!         status: DeviceStatusDto::Online,
//!         datasource_id: cfg.datasource_id.clone(),
//!     }
//! }
//! fn config_interceptor(
//!     req: ConfigRequestDto<MyCfg>,
//!     current: Arc<Config<MyCfg>>,
//! ) -> Config<MyCfg> {
//!     (*current).clone()
//! }
//! ```

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

/// Lightweight HTTP status code type for ESP32 device handlers.
///
/// Mirrors the constants from `axum::http::StatusCode` so that handler
/// function signatures only need to drop `async` when porting between the
/// Linux (Axum) and ESP32 implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(pub u16);

impl StatusCode {
    /// 200 OK — the request succeeded.
    pub const OK: Self = StatusCode(200);
    /// 400 Bad Request — the request payload was invalid.
    pub const BAD_REQUEST: Self = StatusCode(400);
    /// 500 Internal Server Error — an unexpected error occurred.
    pub const INTERNAL_SERVER_ERROR: Self = StatusCode(500);

    /// Returns the numeric HTTP status code.
    pub fn as_u16(self) -> u16 {
        self.0
    }
}
