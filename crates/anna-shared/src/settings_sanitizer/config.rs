// v0.0.643: Settings Sanitizer Config (Phase 219)
// Configuration for sanitizer behavior

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{CaseNormalization, SanitizationType};

/// Sanitizer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizerConfig {
    /// Sanitization type
    pub sanitization_type: SanitizationType,
    /// Case normalization
    pub case_normalization: CaseNormalization,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Trim whitespace
    pub trim: bool,
    /// Remove empty
    pub remove_empty: bool,
}

impl SanitizerConfig {
    /// Create new config
    pub fn new(sanitization_type: SanitizationType) -> Self {
        Self {
            sanitization_type,
            case_normalization: CaseNormalization::None,
            category: None,
            trim: true,
            remove_empty: false,
        }
    }

    /// Set case normalization
    pub fn case_normalization(mut self, case: CaseNormalization) -> Self {
        self.case_normalization = case;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set trim
    pub fn trim(mut self, trim: bool) -> Self {
        self.trim = trim;
        self
    }

    /// Set remove empty
    pub fn remove_empty(mut self, remove: bool) -> Self {
        self.remove_empty = remove;
        self
    }
}

impl Default for SanitizerConfig {
    fn default() -> Self {
        Self::new(SanitizationType::Trim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = SanitizerConfig::new(SanitizationType::Trim);
        assert!(c.trim);
    }

    #[test]
    fn test_config_builder() {
        let c = SanitizerConfig::new(SanitizationType::Full)
            .case_normalization(CaseNormalization::Lower)
            .remove_empty(true);
        assert_eq!(c.case_normalization, CaseNormalization::Lower);
        assert!(c.remove_empty);
    }
}
