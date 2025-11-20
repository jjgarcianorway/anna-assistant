//! Command module organization
//!
//! This module provides all CLI command implementations for annactl.
//! Each command is in its own file for better organization.

// Utility functions shared across commands
mod utils;

// Command modules
mod status;
mod advise;
mod apply;
mod report;
mod doctor;
mod config;
mod setup;
mod rollback;
mod autonomy;
mod wiki;
mod health;
mod dismiss;
mod dismissed;
mod history;
mod update;
mod ignore;

// Re-export all public command functions
pub use status::status;
pub use advise::advise;
pub use apply::apply;
pub use report::report;
pub use doctor::doctor;
pub use config::{config, config_simple};
pub use setup::setup;
pub use rollback::{rollback_bundle, rollback_list, rollback_action, rollback_last};
pub use autonomy::autonomy;
pub use wiki::wiki_cache;
pub use health::health;
pub use dismiss::dismiss;
pub use dismissed::dismissed;
pub use history::history;
pub use update::update;
pub use ignore::ignore;

// Re-export utilities for use by other crates if needed
pub(crate) use utils::{check_and_notify_updates, wrap_text, parse_number_ranges};
