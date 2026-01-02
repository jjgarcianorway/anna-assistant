// v0.0.641: Settings Inspector Stats (Phase 217)
// Statistics tracking for inspections

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::InspectionType;

/// Inspector stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InspectorStats {
    /// Total inspections
    pub total_inspections: usize,
    /// Total findings
    pub total_findings: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl InspectorStats {
    /// Record inspection
    pub fn record(&mut self, inspection_type: InspectionType, finding_count: usize) {
        self.total_inspections += 1;
        self.total_findings += finding_count;
        *self.by_type.entry(inspection_type.to_string()).or_insert(0) += 1;
    }

    /// Average findings
    pub fn average_findings(&self) -> f64 {
        if self.total_inspections == 0 {
            0.0
        } else {
            self.total_findings as f64 / self.total_inspections as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let mut s = InspectorStats::default();
        s.record(InspectionType::Structure, 5);
        assert_eq!(s.total_inspections, 1);
        assert_eq!(s.total_findings, 5);
    }
}
