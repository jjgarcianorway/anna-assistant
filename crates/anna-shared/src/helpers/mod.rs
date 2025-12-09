//! Helper package tracking for Anna dependencies (v0.0.221).
//!
//! Tracks external packages that Anna depends on (e.g., ollama).
//! Distinguishes between packages installed by Anna vs. user-installed.
//!
//! v0.0.28: Initial implementation.
//! v0.0.221: Modularized into domain-focused submodules.

mod detection;
mod persistence;
mod registry;
#[cfg(test)]
mod tests;
mod types;

// Re-export for backwards compatibility
pub use detection::{detect_helper, known_helpers};
pub use persistence::{clear_helpers_store, helpers_store_path, load_helpers, save_helpers};
pub use registry::HelpersRegistry;
pub use types::{HelperPackage, InstallSource};
