// Re-export modules needed for testing
pub mod database;
pub mod router;
pub mod config;

// Re-export AppState for testing
pub use crate::config::AppState; 