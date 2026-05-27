//! Request and response types for the scripting service.
//!
//! Enabled by the `scripting_service_dto` feature of `greenhouse_core`.
//!
//! The scripting service issues tokens that smart devices use to authenticate
//! when posting alerts or accessing automation scripts.
//!
//! # Modules
//!
//! - [`endpoints`] — REST path constants
//! - [`token`] — [`TokenDto`](token::TokenDto) for token exchange

pub mod endpoints;
pub mod token;
