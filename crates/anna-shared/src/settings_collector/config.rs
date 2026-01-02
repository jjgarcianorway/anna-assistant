// v0.0.682: Collector Configuration (Phase 258)
// Configuration for settings collection behavior

use serde::{Deserialize, Serialize};
use crate::settings_collector::types::CollectMode;

/// Collector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Collect mode
    pub mode: CollectMode,
    /// Dedup keys
    pub dedup_keys: bool,
    /// Append suffix
    pub append_suffix: String,
    /// Respect priority
    pub respect_priority: bool,
}

impl CollectorConfig {
    /// Create new config
    pub fn new(mode: CollectMode) -> Self {
        Self {
            mode,
            dedup_keys: true,
            append_suffix: "_".to_string(),
            respect_priority: true,
        }
    }

    /// Set dedup keys
    pub fn dedup_keys(mut self, dedup: bool) -> Self {
        self.dedup_keys = dedup;
        self
    }

    /// Set append suffix
    pub fn append_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.append_suffix = suffix.into();
        self
    }

    /// Set respect priority
    pub fn respect_priority(mut self, respect: bool) -> Self {
        self.respect_priority = respect;
        self
    }
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self::new(CollectMode::Merge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = CollectorConfig::new(CollectMode::Merge);
        assert!(c.dedup_keys);
    }

    #[test]
    fn test_config_builder() {
        let c = CollectorConfig::new(CollectMode::Append)
            .append_suffix("_dup")
            .respect_priority(false);
        assert_eq!(c.append_suffix, "_dup");
        assert!(!c.respect_priority);
    }
}
