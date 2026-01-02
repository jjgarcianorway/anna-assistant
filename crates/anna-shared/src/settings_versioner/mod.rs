// v0.0.660: Settings Versioner (Phase 236)
// Versioner for tracking settings configuration versions

mod version_types;
mod version_config;
mod version_core;
mod versioner;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to maintain the original API
pub use version_types::{VersionScheme, BumpType};
pub use version_config::VersionerConfig;
pub use version_core::{SettingsVersion, VersionResult, VersionerStats};
pub use versioner::SettingsVersioner;
pub use registry::SettingsVersionerRegistry;
pub use utils::{format_versioner_registry, is_versioner_query, versioner_fun_fact};
