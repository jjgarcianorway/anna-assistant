// v0.0.665: Settings Validator Hub Module (Phase 241)
// Central hub for coordinating multiple validators

mod types;
mod validation;
mod validator;
mod hub;
mod registry;

#[cfg(test)]
mod tests;

// Re-export all public items
pub use types::{ValidatorType, ValidationSeverity, HubConfig};
pub use validation::{ValidationIssue, HubValidationResult};
pub use validator::{ValidatorEntry, HubStats};
pub use hub::SettingsValidatorHub;
pub use registry::{ValidatorHubRegistry, format_hub_registry, is_hub_query, hub_fun_fact};
