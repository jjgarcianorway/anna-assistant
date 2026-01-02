// v0.0.695: Settings Folio (Phase 271)
// Folio statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::FolioType;
use super::section::FolioSection;

/// Folio stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FolioStats {
    /// Total sections
    pub total_sections: usize,
    /// Total settings
    pub total_settings: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl FolioStats {
    /// Update from folio
    pub fn update(&mut self, sections: &[FolioSection], folio_type: FolioType) {
        self.total_sections = sections.len();
        self.total_settings = sections.iter().map(|s| s.count()).sum();
        *self.by_type.entry(folio_type.to_string()).or_insert(0) += 1;
    }

    /// Avg settings per section
    pub fn avg_per_section(&self) -> f64 {
        if self.total_sections == 0 { 0.0 } else { self.total_settings as f64 / self.total_sections as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = FolioStats::default();
        let sections = vec![FolioSection::new("s1", "Section", 0)];
        s.update(&sections, FolioType::Active);
        assert_eq!(s.total_sections, 1);
    }
}
