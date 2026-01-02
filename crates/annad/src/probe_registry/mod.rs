//! Probe ID registry - maps probe IDs to actual commands.
//!
//! Extracted from translator.rs (v0.0.164) for modularization.
//! v0.0.405: Domain→probes mapping moved to probe_domain.rs.
//!
//! This module has been split into smaller submodules for better organization:
//! - constants: PROBE_IDS constant
//! - mappings: probe_id_to_command function
//! - dynamic: dynamic probe generation
//! - filters: probe filtering utilities
//! - tests: test module

mod constants;
mod dynamic;
mod filters;
mod mappings;
#[cfg(test)]
mod tests;

// Re-export public API
pub use constants::PROBE_IDS;
pub use dynamic::probe_id_to_command_dynamic;
pub use filters::filter_valid_probes;
pub use mappings::probe_id_to_command;
