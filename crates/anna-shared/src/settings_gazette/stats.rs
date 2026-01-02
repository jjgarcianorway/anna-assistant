// v0.0.704: Gazette Statistics (Phase 280)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::GazetteType;
use super::notice::GazetteNotice;

/// Gazette stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GazetteStats {
    /// Total notices
    pub total_notices: usize,
    /// Urgent notices
    pub urgent_notices: usize,
    /// Total entries
    pub total_entries: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl GazetteStats {
    /// Update from gazette
    pub fn update(&mut self, notices: &[GazetteNotice], gazette_type: GazetteType) {
        self.total_notices = notices.len();
        self.urgent_notices = notices.iter().filter(|n| n.urgent).count();
        *self.by_type.entry(gazette_type.to_string()).or_insert(0) += 1;
    }

    /// Record entry
    pub fn record_entry(&mut self) {
        self.total_entries += 1;
    }

    /// Urgent rate
    pub fn urgent_rate(&self) -> f64 {
        if self.total_notices == 0 { 0.0 } else { self.urgent_notices as f64 / self.total_notices as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = GazetteStats::default();
        let notices = vec![GazetteNotice::new("n1", "Notice", "Content").urgent(true)];
        s.update(&notices, GazetteType::Official);
        assert_eq!(s.total_notices, 1);
        assert_eq!(s.urgent_notices, 1);
    }
}
