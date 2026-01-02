// v0.0.702: Settings Archive V2 (Phase 278)
// Main settings archive implementation

use super::config::ArchiveConfigV2;
use super::record::{ArchiveBox, ArchiveRecord};
use super::stats::ArchiveStatsV2;

/// Settings archive v2
#[derive(Debug, Clone, Default)]
pub struct SettingsArchiveV2 {
    /// Config
    config: ArchiveConfigV2,
    /// Boxes
    boxes: Vec<ArchiveBox>,
    /// Stats
    stats: ArchiveStatsV2,
}

impl SettingsArchiveV2 {
    /// Create new archive
    pub fn new(config: ArchiveConfigV2) -> Self {
        Self {
            config,
            boxes: Vec::new(),
            stats: ArchiveStatsV2::default(),
        }
    }

    /// Add box
    pub fn add_box(&mut self, archive_box: ArchiveBox) {
        self.boxes.push(archive_box);
        self.update_stats();
    }

    /// Get box
    pub fn get_box(&self, id: &str) -> Option<&ArchiveBox> {
        self.boxes.iter().find(|b| b.id == id)
    }

    /// Get box mut
    pub fn get_box_mut(&mut self, id: &str) -> Option<&mut ArchiveBox> {
        self.boxes.iter_mut().find(|b| b.id == id)
    }

    /// Archive record to box
    pub fn archive_to_box(&mut self, box_id: &str, record: ArchiveRecord) -> bool {
        if let Some(archive_box) = self.get_box_mut(box_id) {
            archive_box.add(record);
            self.update_stats();
            true
        } else {
            false
        }
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.boxes, self.config.archive_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ArchiveStatsV2 {
        &self.stats
    }

    /// Box count
    pub fn box_count(&self) -> usize {
        self.boxes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_new() {
        let a = SettingsArchiveV2::new(ArchiveConfigV2::default());
        assert_eq!(a.box_count(), 0);
    }

    #[test]
    fn test_archive_add_box() {
        let mut a = SettingsArchiveV2::new(ArchiveConfigV2::default());
        a.add_box(ArchiveBox::new("b1", "Box 1"));
        assert_eq!(a.box_count(), 1);
    }
}
