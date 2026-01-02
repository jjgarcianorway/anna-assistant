// v0.0.711: Settings Summary Registry (Phase 287)
// Registry for managing multiple settings summaries

use std::collections::HashMap;
use super::summary::SettingsSummary;

/// Summary registry
#[derive(Debug, Clone, Default)]
pub struct SummaryRegistry {
    /// Summaries by ID
    summaries: HashMap<String, SettingsSummary>,
}

impl SummaryRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register summary
    pub fn register(&mut self, id: impl Into<String>, summary: SettingsSummary) {
        self.summaries.insert(id.into(), summary);
    }

    /// Unregister summary
    pub fn unregister(&mut self, id: &str) -> bool {
        self.summaries.remove(id).is_some()
    }

    /// Get summary
    pub fn get(&self, id: &str) -> Option<&SettingsSummary> {
        self.summaries.get(id)
    }

    /// Get summary mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSummary> {
        self.summaries.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.summaries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_summary::types::SummaryConfig;

    #[test]
    fn test_registry_new() {
        let r = SummaryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SummaryRegistry::new();
        r.register("s1", SettingsSummary::new(SummaryConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
