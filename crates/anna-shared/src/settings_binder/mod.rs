// v0.0.652: Settings Binder (Phase 228)
// Binder for connecting settings to runtime objects

mod types;
mod binding;
mod config;
mod stats;
mod binder;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types to maintain API compatibility
pub use types::{BindingType, BindingState};
pub use binding::{BindingDef, BindingResult};
pub use config::BinderConfig;
pub use stats::BinderStats;
pub use binder::{SettingsBinder, SettingsBinderRegistry};
pub use helpers::{format_binder_registry, is_binder_query, binder_fun_fact};
