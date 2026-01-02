//! Configuration management for annad.
//!
//! Loads settings from /etc/anna/config.toml or uses defaults.
//! v0.0.76: Added model registry with domain-specific specialist support.
//! v0.0.162: Model registry extracted to separate module.

mod defaults;
mod loading;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types and constants to maintain API compatibility
pub use types::{
    BudgetConfig, Config, DaemonConfig, LlmConfig, ModelRegistryConfig, CONFIG_PATH,
    DEFAULT_CONFIG_PATH,
};
