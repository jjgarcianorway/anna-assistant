//! Service configuration recipes (v0.0.214).
//!
//! Safe systemd service management.
//!
//! v0.0.214: Modularized into domain-focused submodules.
//!
//! # Supported Operations
//! - Start/stop/restart services
//! - Enable/disable at boot
//! - Check status
//!
//! # Safety
//! - All changes require user confirmation
//! - Only manages user-requested services
//! - No system-critical services can be disabled

pub mod catalog;
pub mod prompt;
pub mod recipe;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export for backwards compatibility
pub use catalog::{find_service, known_services};
pub use prompt::confirmation_prompt;
pub use recipe::ServiceRecipe;
pub use types::{ServiceAction, ServiceCategory, ServiceRisk};
