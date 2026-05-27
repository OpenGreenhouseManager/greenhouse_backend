//! Shared types and smart-device interface for the OpenGreenhouseManager backend.
//!
//! `greenhouse_core` provides two categories of items:
//!
//! 1. **Data Transfer Objects (DTOs)** — plain Rust structs and enums that are
//!    serialised over HTTP between the individual microservices and between services
//!    and ESP32 firmware. Each group of DTOs is gated behind a feature flag so
//!    consumers only compile what they need.
//!
//! 2. **Smart-device interface** — a higher-level abstraction built on top of
//!    [Axum] that turns a set of async handler functions into a fully-wired HTTP
//!    server exposing `/read`, `/write`, `/config`, `/status`, and `/activate`
//!    endpoints. Used by the example device binaries and by `greenhouse_esp32`.
//!
//! # Feature flags
//!
//! All modules are gated; the `default` feature set enables everything listed below.
//!
//! | Feature | Enables |
//! |---------|---------|
//! | `auth_service_dto` | [`auth_service_dto`] — user authentication DTOs |
//! | `data_storage_service_dto` | [`data_storage_service_dto`] — alert and diary DTOs |
//! | `device_service_dto` | [`device_service_dto`] — device registry DTOs |
//! | `scripting_service_dto` | [`scripting_service_dto`] — scripting token DTOs |
//! | `smart_device_dto` | [`smart_device_dto`] — sensor data types shared by devices |
//! | `smart_device_interface` | [`smart_device_interface`] — Axum router builder (pulls in `axum`, `reqwest`, `tracing`) |
//! | `error_handling` | [`http_error`] — `HttpErrorResponse` and `HttpErrorMapping` trait |
//! | `api_web_dto` | Reserved — no items yet |
//! | `api_script_dto` | Reserved — no items yet |
//!
//! [Axum]: https://docs.rs/axum
//!
//! # `greenhouse_esp32`
//!
//! A sibling crate, `greenhouse_esp32`, mirrors the `smart_device_interface` API for
//! ESP32 targets using `esp-idf-svc`. Handler functions are synchronous (`fn`, not
//! `async fn`) and configuration is persisted to NVS rather than a JSON file. It is
//! not published to crates.io but lives in the same repository.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "auth_service_dto")]
#[cfg_attr(docsrs, doc(cfg(feature = "auth_service_dto")))]
pub mod auth_service_dto;

#[cfg(feature = "data_storage_service_dto")]
#[cfg_attr(docsrs, doc(cfg(feature = "data_storage_service_dto")))]
pub mod data_storage_service_dto;

#[cfg(feature = "device_service_dto")]
#[cfg_attr(docsrs, doc(cfg(feature = "device_service_dto")))]
pub mod device_service_dto;

#[cfg(feature = "scripting_service_dto")]
#[cfg_attr(docsrs, doc(cfg(feature = "scripting_service_dto")))]
pub mod scripting_service_dto;

#[cfg(feature = "smart_device_dto")]
#[cfg_attr(docsrs, doc(cfg(feature = "smart_device_dto")))]
pub mod smart_device_dto;

#[cfg(feature = "smart_device_interface")]
#[cfg_attr(docsrs, doc(cfg(feature = "smart_device_interface")))]
pub mod smart_device_interface;

#[cfg(feature = "error_handling")]
#[cfg_attr(docsrs, doc(cfg(feature = "error_handling")))]
pub mod http_error;
