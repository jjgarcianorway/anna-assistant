// v0.0.641: Settings Inspector (Phase 217)
// Main inspector implementation

use super::config::InspectorConfig;
use super::result::InspectionResult;
use super::stats::InspectorStats;

/// Settings inspector
#[derive(Debug, Clone, Default)]
pub struct SettingsInspector {
    /// Config
    config: InspectorConfig,
    /// Results
    results: Vec<InspectionResult>,
    /// Stats
    stats: InspectorStats,
}

impl SettingsInspector {
    /// Create new inspector
    pub fn new(config: InspectorConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: InspectorStats::default(),
        }
    }

    /// Inspect
    pub fn inspect(&mut self, id: impl Into<String>) -> InspectionResult {
        let result = InspectionResult::new(id, self.config.inspection_type);
        self.stats.record(self.config.inspection_type, 0);
        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[InspectionResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &InspectorStats {
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
    use crate::settings_inspector::types::InspectionType;

    #[test]
    fn test_inspector_new() {
        let i = SettingsInspector::new(InspectorConfig::new(InspectionType::Structure));
        assert_eq!(i.result_count(), 0);
    }

    #[test]
    fn test_inspector_inspect() {
        let mut i = SettingsInspector::new(InspectorConfig::new(InspectionType::Structure));
        i.inspect("i1");
        assert_eq!(i.result_count(), 1);
    }
}
