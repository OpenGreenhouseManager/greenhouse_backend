//! Request and response types for the data storage service.
//!
//! Enabled by the `data_storage_service_dto` feature of `greenhouse_core`.
//!
//! The data storage service persists two categories of data:
//!
//! - **Alerts** — threshold violations or diagnostic events raised by smart devices.
//! - **Diary entries** — freeform log entries with optional tags, used for manual
//!   observations about greenhouse conditions.
//!
//! # Modules
//!
//! - [`alert_dto`] — alert CRUD, aggregation, and query types
//! - [`diary_dtos`] — diary entry CRUD types

pub mod alert_dto;
pub mod diary_dtos;
