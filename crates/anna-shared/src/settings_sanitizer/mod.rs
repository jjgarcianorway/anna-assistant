// v0.0.643: Settings Sanitizer Module (Phase 219)
// Sanitizer for cleaning and validating settings values

mod types;
mod config;
mod result;
mod stats;
mod sanitizer;
mod registry;

// Re-export types
pub use types::{SanitizationType, CaseNormalization};
pub use config::SanitizerConfig;
pub use result::SanitizationResult;
pub use stats::SanitizerStats;
pub use sanitizer::SettingsSanitizer;
pub use registry::{
    SettingsSanitizerRegistry,
    format_sanitizer_registry,
    is_sanitizer_query,
    sanitizer_fun_fact,
};
