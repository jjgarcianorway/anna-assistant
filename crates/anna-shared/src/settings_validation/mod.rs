// v0.0.557: Settings Validation Module
// Validates settings and detects conflicts or invalid configurations

mod types;
mod validator;
mod formatting;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve API
pub use types::{
    ValidationCategory,
    ValidationIssue,
    ValidationResult,
    ValidationSeverity,
};

pub use validator::{
    SettingsValidator,
    validate_settings,
};

pub use formatting::{
    format_validation_result,
    settings_validation_fun_fact,
};
