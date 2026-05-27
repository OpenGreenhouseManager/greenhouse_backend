//! Request and response types for the device registry service.
//!
//! Enabled by the `device_service_dto` feature of `greenhouse_core`.
//!
//! The device service maintains a registry of smart devices connected to the
//! greenhouse network, records their time-series sensor readings, and exposes
//! management operations.
//!
//! # Modules
//!
//! - [`endpoints`] — REST path constants
//! - [`get_device`] — device listing and detail types
//! - [`post_device`] — device registration request
//! - [`put_device`] — device update request
//! - [`get_timeseries`] — time-series sensor data types
//! - [`operations`] — available device operations
//! - [`query`] — time-range query parameters for sensor data

pub mod endpoints;
pub mod get_device;
pub mod get_timeseries;
pub mod operations;
pub mod post_device;
pub mod put_device;
pub mod query;
