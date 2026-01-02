// v0.0.709: Digest Statistics (Phase 285)
// Statistics tracking for digest generation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::DigestFormat;
use super::section::DigestSection;

/// Digest stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DigestStats {
    /// Total sections
    pub total_sections: usize,
    /// Total items
    pub total_items: usize,
    /// Highlighted items
    pub highlighted: usize,
    /// By format
    pub by_format: HashMap<String, usize>,
}

impl DigestStats {
    /// Update from digest
    pub fn update(&mut self, sections: &[DigestSection], format: DigestFormat) {
        self.total_sections = sections.len();
        self.total_items = sections.iter().map(|s| s.item_count()).sum();
        self.highlighted = sections.iter()
            .flat_map(|s| &s.items)
            .filter(|i| i.highlight)
            .count();
        *self.by_format.entry(format.to_string()).or_insert(0) += 1;
    }

    /// Highlight rate
    pub fn highlight_rate(&self) -> f64 {
        if self.total_items == 0 { 0.0 } else { self.highlighted as f64 / self.total_items as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_digest::section::DigestItem;

    #[test]
    fn test_stats_update() {
        let mut s = DigestStats::default();
        let mut section = DigestSection::new("s1", "Section", 1);
        section.add(DigestItem::new("key", "value").highlight(true));
        s.update(&[section], DigestFormat::Summary);
        assert_eq!(s.total_sections, 1);
        assert_eq!(s.highlighted, 1);
    }
}
