//! Axum-based HTTP server builder for smart devices.
//!
//! Enabled by the `smart_device_interface` feature of `greenhouse_core`.
//!
//! A *smart device* in OpenGreenhouseManager is a small HTTP server that exposes a
//! standardised set of endpoints. This module provides [`device_builder::DeviceBuilder`], which
//! takes a set of async handler functions and wires them into an [`axum::Router`]
//! ready to be served.
//!
//! # Device archetypes
//!
//! | Archetype | Constructor | Endpoints |
//! |-----------|-------------|-----------|
//! | Output (sensor) | [`device_builder::DeviceBuilder::new_output_device`] | `/read`, `/status`, `/config`, `/activate` |
//! | Input (actuator) | [`device_builder::DeviceBuilder::new_input_device`] | `/write`, `/status`, `/config`, `/activate` |
//! | Hybrid | [`device_builder::DeviceBuilder::new_hybrid_device`] | all five endpoints |
//!
//! # Generic parameter `T`
//!
//! The type parameter `T` is the device's *additional configuration* — any fields
//! beyond the standard `port` and `datasource_id`. It must implement
//! `Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static`.
//!
//! # Example
//!
//! See the workspace-level `examples/` crate for complete runnable examples
//! (`input_output_int_saver`, `input_alert_trigger`, `periodic_alert`).
//!
//! For a minimal outline:
//!
//! ```no_run
//! # use greenhouse_core::smart_device_interface::{
//! #     device_builder::DeviceBuilder,
//! #     output_device::init_output_router,
//! # };
//! # use greenhouse_core::smart_device_dto::{
//! #     Type, config::TypeOption,
//! #     status::{DeviceStatusDto, DeviceStatusResponseDto},
//! # };
//! # use greenhouse_core::smart_device_interface::config::Config;
//! # use std::sync::Arc;
//! # #[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
//! # struct MyConfig {}
//! async fn read(cfg: Arc<Config<MyConfig>>) -> Type {
//!     Type::Number(42.0)
//! }
//! async fn status(cfg: Arc<Config<MyConfig>>) -> DeviceStatusResponseDto {
//!     DeviceStatusResponseDto {
//!         status: DeviceStatusDto::Online,
//!         datasource_id: cfg.datasource_id.clone(),
//!     }
//! }
//! async fn config_interceptor(
//!     req: greenhouse_core::smart_device_dto::config::ConfigRequestDto<MyConfig>,
//!     current: Arc<Config<MyConfig>>,
//! ) -> Config<MyConfig> {
//!     (*current).clone()
//! }
//!
//! let builder = DeviceBuilder::new_output_device(
//!     read, status, config_interceptor, TypeOption::Number,
//! ).unwrap();
//!
//! let router = init_output_router(builder);
//! // Serve `router` with axum::serve(...)
//! ```

pub mod config;
pub mod device_builder;
pub mod device_service;
mod error;
mod handler;
pub mod hybrid_device;
pub mod input_device;
pub mod output_device;

pub use self::error::{Error, Result};
