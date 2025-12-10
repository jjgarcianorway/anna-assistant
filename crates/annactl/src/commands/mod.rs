//! Command handlers for annactl (v0.0.237).
//! v0.0.144: Simplified - removed unnecessary flags, natural language for everything.
//! v0.0.205: Modularized into domain-focused submodules.
//! v0.0.237: Added config command handler for natural language settings.

pub mod config;
mod feedback;
mod handlers;
mod repl;

// Re-export all handlers
#[allow(unused_imports)]
pub use config::{show_config_status, try_handle_config, ConfigResult};
pub use handlers::{handle_request, handle_reset, handle_stats, handle_status, handle_uninstall};
pub use repl::handle_repl;
