//! Request and response types for the authentication service.
//!
//! Enabled by the `auth_service_dto` feature of `greenhouse_core`.
//!
//! # Modules
//!
//! - [`endpoints`] — REST path constants
//! - [`login`] — [`LoginRequestDto`](login::LoginRequestDto) / [`LoginResponseDto`](login::LoginResponseDto)
//! - [`register`] — guest registration with a one-time token
//! - [`register_admin`] — initial admin account creation
//! - [`token`] — token validation request and response
//! - [`user_token`] — decoded JWT claims
//! - [`user_preferences`] — dashboard and alert preference DTOs
//! - [`generate_one_time_token`] — admin-issued invitation tokens

pub mod endpoints;
pub mod generate_one_time_token;
pub mod login;
pub mod register;
pub mod register_admin;
pub mod token;
pub mod user_preferences;
pub mod user_token;
