//! Alert data transfer objects.
//!
//! Alerts are discrete events — threshold crossings, sensor faults, or other
//! noteworthy conditions — raised by smart devices and stored by the data
//! storage service.
//!
//! # Modules
//!
//! - [`endpoints`] — REST path constants
//! - [`alert`] — [`AlertDto`](alert::AlertDto), [`AlertsDto`](alert::AlertsDto), [`Severity`](alert::Severity)
//! - [`post_create_alert`] — [`CreateAlertDto`](post_create_alert::CreateAlertDto)
//! - [`get_aggrigated_alert`] — aggregated alert summary types
//! - [`query`] — filter and time-range query parameters

pub mod alert;
pub mod endpoints;
pub mod get_aggrigated_alert;
pub mod post_create_alert;
pub mod query;
