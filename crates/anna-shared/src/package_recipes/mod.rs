//! Package installation recipes (v0.0.230).
//!
//! v0.0.98: Safe package installation with multi-manager support.
//!
//! # Supported Package Managers
//! - pacman (Arch Linux)
//! - apt (Debian/Ubuntu)
//! - dnf (Fedora)
//! - flatpak (universal)
//!
//! # Safety
//! - All installs require user confirmation
//! - Uses system package managers (no untrusted sources)
//! - Transaction support for multi-package installs
//!
//! v0.0.230: Modularized into domain-focused submodules.

mod catalog;
mod search;
#[cfg(test)]
mod tests;
mod types;

// Re-export for backwards compatibility
pub use catalog::common_packages;
pub use search::{confirmation_prompt, find_recipe};
pub use types::{PackageCategory, PackageManager, PackageRecipe};
