//! Deterministic answer functions (v0.0.171).
//!
//! This module contains deterministic answer functions split into logical submodules.
//! Each submodule handles a specific category of system queries.

pub mod meta;
pub mod services;
pub mod system;

// Re-export all public functions from submodules
pub use meta::*;
pub use services::*;
pub use system::*;
