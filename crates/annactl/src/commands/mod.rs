//! Command handlers for annactl (v0.0.205).
//! v0.0.144: Simplified - removed unnecessary flags, natural language for everything.
//! v0.0.205: Modularized into domain-focused submodules.

mod feedback;
mod handlers;
mod repl;

// Re-export all handlers
pub use handlers::{handle_request, handle_stats, handle_status, handle_uninstall};
pub use repl::handle_repl;
