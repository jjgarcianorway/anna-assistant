// v0.0.702: Settings Archive V2 (Phase 278)
// Archive statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ArchiveTypeV2;
use super::record::ArchiveBox;

/// Archive stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchiveStatsV2 {
    /// Total boxes
    pub total_boxes: usize,
    /// Total records
    pub total_records: usize,
    /// Sealed boxes
    pub sealed_boxes: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ArchiveStatsV2 {
    /// Update from archive
    pub fn update(&mut self, boxes: &[ArchiveBox], archive_type: ArchiveTypeV2) {
        self.total_boxes = boxes.len();
        self.total_records = boxes.iter().map(|b| b.record_count()).sum();
        self.sealed_boxes = boxes.iter().filter(|b| b.sealed).count();
        *self.by_type.entry(archive_type.to_string()).or_insert(0) += 1;
    }

    /// Sealed rate
    pub fn sealed_rate(&self) -> f64 {
        if self.total_boxes == 0 { 0.0 } else { self.sealed_boxes as f64 / self.total_boxes as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = ArchiveStatsV2::default();
        let boxes = vec![ArchiveBox::new("b1", "Box")];
        s.update(&boxes, ArchiveTypeV2::Cold);
        assert_eq!(s.total_boxes, 1);
    }
}
