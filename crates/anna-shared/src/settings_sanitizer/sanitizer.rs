// v0.0.643: Settings Sanitizer (Phase 219)
// Core sanitizer implementation

use super::config::SanitizerConfig;
use super::result::SanitizationResult;
use super::stats::SanitizerStats;
use super::types::CaseNormalization;

/// Settings sanitizer
#[derive(Debug, Clone, Default)]
pub struct SettingsSanitizer {
    /// Config
    config: SanitizerConfig,
    /// Results
    results: Vec<SanitizationResult>,
    /// Stats
    stats: SanitizerStats,
}

impl SettingsSanitizer {
    /// Create new sanitizer
    pub fn new(config: SanitizerConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: SanitizerStats::default(),
        }
    }

    /// Sanitize value
    pub fn sanitize(&mut self, value: impl Into<String>) -> SanitizationResult {
        let original = value.into();
        let mut sanitized = original.clone();
        let mut operations = Vec::new();

        if self.config.trim {
            let trimmed = sanitized.trim().to_string();
            if trimmed != sanitized {
                operations.push("trim".to_string());
                sanitized = trimmed;
            }
        }

        match self.config.case_normalization {
            CaseNormalization::Lower => {
                let lower = sanitized.to_lowercase();
                if lower != sanitized {
                    operations.push("lowercase".to_string());
                    sanitized = lower;
                }
            }
            CaseNormalization::Upper => {
                let upper = sanitized.to_uppercase();
                if upper != sanitized {
                    operations.push("uppercase".to_string());
                    sanitized = upper;
                }
            }
            _ => {}
        }

        let changed = original != sanitized;
        self.stats.record(self.config.sanitization_type, changed);

        let mut result = SanitizationResult::new(original, sanitized);
        for op in operations {
            result.add_operation(op);
        }
        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[SanitizationResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &SanitizerStats {
        &self.stats
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{SanitizationType, CaseNormalization};
    use super::super::config::SanitizerConfig;

    #[test]
    fn test_sanitizer_new() {
        let s = SettingsSanitizer::new(SanitizerConfig::new(SanitizationType::Trim));
        assert_eq!(s.result_count(), 0);
    }

    #[test]
    fn test_sanitizer_sanitize_trim() {
        let mut s = SettingsSanitizer::new(SanitizerConfig::new(SanitizationType::Trim));
        let r = s.sanitize("  test  ");
        assert!(r.was_changed());
        assert_eq!(r.sanitized, "test");
    }

    #[test]
    fn test_sanitizer_sanitize_case() {
        let mut s = SettingsSanitizer::new(
            SanitizerConfig::new(SanitizationType::NormalizeCase)
                .case_normalization(CaseNormalization::Lower)
        );
        let r = s.sanitize("TEST");
        assert!(r.was_changed());
        assert_eq!(r.sanitized, "test");
    }
}
