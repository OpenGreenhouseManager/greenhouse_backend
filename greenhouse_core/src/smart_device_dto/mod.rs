//! Protocol-level types exchanged between the device service and smart devices.
//!
//! Enabled by the `smart_device_dto` feature of `greenhouse_core`.
//!
//! Smart devices (sensors and actuators) communicate with the greenhouse backend
//! over HTTP using a small set of endpoints. This module contains the DTOs for
//! those endpoints, as well as the core [`Type`] enum used to represent sensor
//! readings and actuator commands uniformly.
//!
//! # Modules
//!
//! - [`endpoints`] — REST path constants (`/read`, `/write`, `/status`, `/config`, `/activate`)
//! - [`read`] — response type for the `/read` endpoint
//! - [`mod@write`] — request type for the `/write` endpoint
//! - [`status`] — device status types
//! - [`config`] — device configuration types
//! - [`activation`] — request type for the `/activate` endpoint

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod activation;
pub mod config;
pub mod endpoints;
pub mod read;
pub mod status;
pub mod write;

/// A value exchanged between a smart device and the greenhouse backend.
///
/// Used as the payload of [`read::ReadResponseDto`] and [`write::WriteRequestDto`].
///
/// # Example
///
/// ```
/// # use greenhouse_core::smart_device_dto::{Type, Measurement};
/// let temp = Type::Measurement(Measurement {
///     value: 23.5,
///     unit: "°C".to_string(),
/// });
/// let json = serde_json::to_string(&temp).unwrap();
/// assert!(json.contains("23.5"));
/// ```
#[derive(Serialize, Deserialize)]
pub enum Type {
    /// A raw numeric value (e.g. voltage, raw ADC counts).
    Number(f64),
    /// A boolean state (e.g. relay open/closed).
    Boolean(bool),
    /// A structured key-value reading (e.g. multi-channel sensor output).
    Object(HashMap<String, Type>),
    /// A physical measurement with a value and SI unit.
    Measurement(Measurement),
    /// A streaming data source — no discrete value to return.
    Stream,
    /// No value available (device not ready, error state, etc.).
    None,
}

/// A physical measurement pairing a numeric value with its unit.
///
/// # Example
///
/// ```
/// # use greenhouse_core::smart_device_dto::Measurement;
/// let m = Measurement { value: 65.2, unit: "%RH".to_string() };
/// assert_eq!(m.unit, "%RH");
/// ```
#[derive(Serialize, Deserialize)]
pub struct Measurement {
    /// The measured quantity.
    pub value: f64,
    /// SI or custom unit string (e.g. `"°C"`, `"hPa"`, `"%RH"`).
    pub unit: String,
}
