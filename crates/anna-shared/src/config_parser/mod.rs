//! Natural language config command parser (v0.0.239).
//!
//! Parses user requests to change Anna's settings, like:
//! - "disable learning mode"
//! - "enable auto-confirm for low risk"
//! - "make Anna more casual"
//! - "hide internal communications"
//! - "set verbosity to detailed"
//! - "my email is user@example.com"
//! - "notify me at user@example.com"
//!
//! v0.0.239: Added email setup via natural language.

mod parser;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use parser::{is_config_request, parse_config_request};
pub use types::ConfigChange;
