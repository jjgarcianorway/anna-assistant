// v0.0.779: Settings Apiary - Stats (Phase 355)
// Apiary statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ApiaryType;
use super::hive::ApiaryHive;

/// Apiary stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiaryStats {
    /// Total hives
    pub total_hives: usize,
    /// Productive hives
    pub productive: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ApiaryStats {
    /// Update from hives
    pub fn update(&mut self, hives: &[ApiaryHive], apiary_type: ApiaryType) {
        self.total_hives = hives.len();
        self.productive = hives.iter().filter(|h| h.productive).count();
        *self.by_type.entry(apiary_type.to_string()).or_insert(0) += 1;
    }

    /// Productivity rate
    pub fn productivity_rate(&self) -> f64 {
        if self.total_hives == 0 { 0.0 } else { self.productive as f64 / self.total_hives as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = ApiaryStats::default();
        let hive = ApiaryHive::new("h1", "Title", "Content");
        s.update(&[hive], ApiaryType::Honey);
        assert_eq!(s.total_hives, 1);
        assert_eq!(s.productive, 1);
    }
}
