// v0.0.641: Settings Inspector Result (Phase 217)
// Result of an inspection operation

use serde::{Deserialize, Serialize};

use super::finding::InspectionFinding;
use super::types::InspectionType;

/// Inspection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResult {
    /// ID
    pub id: String,
    /// Inspection type
    pub inspection_type: InspectionType,
    /// Findings
    pub findings: Vec<InspectionFinding>,
    /// Timestamp
    pub timestamp: u64,
    /// Duration ms
    pub duration_ms: u64,
}

impl InspectionResult {
    /// Create new result
    pub fn new(id: impl Into<String>, inspection_type: InspectionType) -> Self {
        Self {
            id: id.into(),
            inspection_type,
            findings: Vec::new(),
            timestamp: 0,
            duration_ms: 0,
        }
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    /// Set duration
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Add finding
    pub fn add_finding(&mut self, finding: InspectionFinding) {
        self.findings.push(finding);
    }

    /// Finding count
    pub fn finding_count(&self) -> usize {
        self.findings.len()
    }

    /// Has findings
    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_settings::SettingsCategory;

    #[test]
    fn test_result_new() {
        let r = InspectionResult::new("r1", InspectionType::Structure);
        assert_eq!(r.finding_count(), 0);
    }

    #[test]
    fn test_result_findings() {
        let mut r = InspectionResult::new("r1", InspectionType::Structure);
        r.add_finding(InspectionFinding::new("f1", SettingsCategory::Privacy, "key", "missing"));
        assert!(r.has_findings());
    }
}
