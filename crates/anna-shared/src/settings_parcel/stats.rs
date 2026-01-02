// v0.0.757: Settings Parcel - Stats (Phase 333)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ParcelType;
use super::title::ParcelTitle;

/// Parcel stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParcelStats {
    /// Total titles
    pub total_titles: usize,
    /// Cleared titles
    pub cleared: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ParcelStats {
    /// Update from titles
    pub fn update(&mut self, titles: &[ParcelTitle], parcel_type: ParcelType) {
        self.total_titles = titles.len();
        self.cleared = titles.iter().filter(|t| t.cleared).count();
        *self.by_type.entry(parcel_type.to_string()).or_insert(0) += 1;
    }

    /// Cleared rate
    pub fn cleared_rate(&self) -> f64 {
        if self.total_titles == 0 { 0.0 } else { self.cleared as f64 / self.total_titles as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = ParcelStats::default();
        let title = ParcelTitle::new("t1", "Title", "Content");
        s.update(&[title], ParcelType::FeeSimple);
        assert_eq!(s.total_titles, 1);
        assert_eq!(s.cleared, 1);
    }
}
