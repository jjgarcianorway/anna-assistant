// v0.0.762: Settings Field Stats (Phase 338)
// Field statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::FieldType;
use super::crop::FieldCrop;

/// Field stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldStats {
    /// Total crops
    pub total_crops: usize,
    /// Yielded crops
    pub yielded: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl FieldStats {
    /// Update from crops
    pub fn update(&mut self, crops: &[FieldCrop], field_type: FieldType) {
        self.total_crops = crops.len();
        self.yielded = crops.iter().filter(|c| c.yielded).count();
        *self.by_type.entry(field_type.to_string()).or_insert(0) += 1;
    }

    /// Yield rate
    pub fn yield_rate(&self) -> f64 {
        if self.total_crops == 0 { 0.0 } else { self.yielded as f64 / self.total_crops as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = FieldStats::default();
        let crop = FieldCrop::new("c1", "Title", "Content");
        s.update(&[crop], FieldType::Arable);
        assert_eq!(s.total_crops, 1);
        assert_eq!(s.yielded, 1);
    }
}
