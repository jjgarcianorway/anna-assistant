//! REPL Greeting - Stats-based personalized greetings (v0.0.413).
//! v0.0.463: Enhanced with error announcements per VISION.md Phase 29.
//!
//! Generates a greeting for the REPL that uses real ticket stats
//! to create a personalized "IT department" welcome.

mod builder;
mod helpers;
mod renderer;
mod session;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use session::SessionContext;
pub use types::{ReplGreeting, SystemError, SystemStatus};
