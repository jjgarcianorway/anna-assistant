// v0.0.688: Settings Validator Module (Phase 264)
// Validate settings against rules and constraints

mod types;
mod config;
mod rule;
mod issue;
mod result;
mod stats;
mod validator;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public items
pub use types::{ValidationType, ValidationSeverity};
pub use config::ValidatorConfig;
pub use rule::ValidationRule;
pub use issue::ValidationIssue;
pub use result::ValidationResult;
pub use stats::ValidatorStats;
pub use validator::SettingsValidator;
pub use registry::ValidatorRegistry;
pub use utils::{format_validator_registry, is_validator_query, validator_fun_fact};
