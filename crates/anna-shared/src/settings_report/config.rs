// v0.0.712: Settings Report Config (Phase 288)
// Report configuration

use serde::{Deserialize, Serialize};
use super::types::{ReportType, ReportFrequency};

/// Report config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Name
    pub name: String,
    /// Report type
    pub report_type: ReportType,
    /// Frequency
    pub frequency: ReportFrequency,
    /// Max sections
    pub max_sections: usize,
}

impl ReportConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            report_type: ReportType::Status,
            frequency: ReportFrequency::OnDemand,
            max_sections: 50,
        }
    }

    /// Set type
    pub fn report_type(mut self, rt: ReportType) -> Self {
        self.report_type = rt;
        self
    }

    /// Set frequency
    pub fn frequency(mut self, f: ReportFrequency) -> Self {
        self.frequency = f;
        self
    }

    /// Set max sections
    pub fn max_sections(mut self, max: usize) -> Self {
        self.max_sections = max;
        self
    }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ReportConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ReportConfig::new("test")
            .report_type(ReportType::Compliance)
            .frequency(ReportFrequency::Monthly);
        assert_eq!(c.report_type, ReportType::Compliance);
        assert_eq!(c.frequency, ReportFrequency::Monthly);
    }
}
