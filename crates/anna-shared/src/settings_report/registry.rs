// v0.0.712: Settings Report Registry (Phase 288)
// Report registry for managing multiple reports

use std::collections::HashMap;
use super::report::SettingsReport;

/// Report registry
#[derive(Debug, Clone, Default)]
pub struct ReportRegistry {
    /// Reports by ID
    reports: HashMap<String, SettingsReport>,
}

impl ReportRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register report
    pub fn register(&mut self, id: impl Into<String>, report: SettingsReport) {
        self.reports.insert(id.into(), report);
    }

    /// Unregister report
    pub fn unregister(&mut self, id: &str) -> bool {
        self.reports.remove(id).is_some()
    }

    /// Get report
    pub fn get(&self, id: &str) -> Option<&SettingsReport> {
        self.reports.get(id)
    }

    /// Get report mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsReport> {
        self.reports.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.reports.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_report::ReportConfig;

    #[test]
    fn test_registry_new() {
        let r = ReportRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ReportRegistry::new();
        r.register("r1", SettingsReport::new(ReportConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
