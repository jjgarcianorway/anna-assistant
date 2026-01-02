// v0.0.645: Settings Normalizer Core (Phase 221)
// Core normalization engine

use super::config::NormalizerConfig;
use super::result::NormalizationResult;
use super::stats::NormalizerStats;
use super::types::{NormalizationRule, NormalizationType};

/// Settings normalizer
#[derive(Debug, Clone, Default)]
pub struct SettingsNormalizer {
    /// Config
    config: NormalizerConfig,
    /// Results
    results: Vec<NormalizationResult>,
    /// Stats
    stats: NormalizerStats,
}

impl SettingsNormalizer {
    /// Create new normalizer
    pub fn new(config: NormalizerConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: NormalizerStats::default(),
        }
    }

    /// Normalize value
    pub fn normalize(&mut self, value: impl Into<String>) -> NormalizationResult {
        let original = value.into();

        if !self.config.enabled {
            let result = NormalizationResult::new(
                original.clone(),
                original,
                self.config.normalization_type,
                NormalizationRule::None,
            );
            self.results.push(result.clone());
            return result;
        }

        let normalized = self.apply_rule(&original);
        self.stats.record(
            self.config.normalization_type,
            self.config.rule,
            normalized != original,
        );

        let result = NormalizationResult::new(
            original,
            normalized,
            self.config.normalization_type,
            self.config.rule,
        );
        self.results.push(result.clone());
        result
    }

    /// Apply normalization rule
    fn apply_rule(&self, value: &str) -> String {
        match self.config.rule {
            NormalizationRule::None => value.to_string(),
            NormalizationRule::Lowercase => value.to_lowercase(),
            NormalizationRule::Uppercase => value.to_uppercase(),
            NormalizationRule::Trim => value.trim().to_string(),
            NormalizationRule::Canonical => {
                // Canonical: trim + lowercase
                value.trim().to_lowercase()
            }
        }
    }

    /// Get results
    pub fn results(&self) -> &[NormalizationResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &NormalizerStats {
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
