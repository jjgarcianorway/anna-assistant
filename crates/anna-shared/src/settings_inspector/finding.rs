// v0.0.641: Settings Inspector Finding (Phase 217)
// Individual inspection finding

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Inspection finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionFinding {
    /// Finding ID
    pub id: String,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Finding type
    pub finding_type: String,
    /// Description
    pub description: String,
    /// Severity
    pub severity: String,
}

impl InspectionFinding {
    /// Create new finding
    pub fn new(
        id: impl Into<String>,
        category: SettingsCategory,
        key: impl Into<String>,
        finding_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            key: key.into(),
            finding_type: finding_type.into(),
            description: String::new(),
            severity: "info".to_string(),
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set severity
    pub fn severity(mut self, sev: impl Into<String>) -> Self {
        self.severity = sev.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_new() {
        let f = InspectionFinding::new("f1", SettingsCategory::Privacy, "key", "missing");
        assert_eq!(f.severity, "info");
    }

    #[test]
    fn test_finding_builder() {
        let f = InspectionFinding::new("f1", SettingsCategory::Privacy, "key", "missing")
            .severity("warning");
        assert_eq!(f.severity, "warning");
    }
}
