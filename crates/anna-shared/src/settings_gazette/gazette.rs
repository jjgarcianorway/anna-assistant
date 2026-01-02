// v0.0.704: Settings Gazette (Phase 280)

use super::config::GazetteConfig;
use super::notice::{GazetteNotice, GazetteEntry};
use super::types::GazetteStatus;
use super::stats::GazetteStats;

/// Settings gazette
#[derive(Debug, Clone, Default)]
pub struct SettingsGazette {
    /// Config
    config: GazetteConfig,
    /// Notices
    notices: Vec<GazetteNotice>,
    /// Entries
    entries: Vec<GazetteEntry>,
    /// Status
    status: GazetteStatus,
    /// Stats
    stats: GazetteStats,
}

impl SettingsGazette {
    /// Create new gazette
    pub fn new(config: GazetteConfig) -> Self {
        Self {
            config,
            notices: Vec::new(),
            entries: Vec::new(),
            status: GazetteStatus::Draft,
            stats: GazetteStats::default(),
        }
    }

    /// Add notice
    pub fn add_notice(&mut self, notice: GazetteNotice) -> bool {
        if self.notices.len() >= self.config.max_notices {
            return false;
        }
        self.notices.push(notice);
        self.update_stats();
        true
    }

    /// Get notice
    pub fn get_notice(&self, id: &str) -> Option<&GazetteNotice> {
        self.notices.iter().find(|n| n.id == id)
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: GazetteEntry) {
        self.entries.push(entry);
        self.stats.record_entry();
    }

    /// Get entries for notice
    pub fn get_entries(&self, notice_id: &str) -> Vec<&GazetteEntry> {
        self.entries.iter().filter(|e| e.notice_id == notice_id).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.notices, self.config.gazette_type);
    }

    /// Submit for review
    pub fn review(&mut self) {
        self.status = GazetteStatus::Review;
    }

    /// Publish
    pub fn publish(&mut self) {
        self.status = GazetteStatus::Published;
    }

    /// Supersede
    pub fn supersede(&mut self) {
        self.status = GazetteStatus::Superseded;
    }

    /// Get status
    pub fn status(&self) -> GazetteStatus {
        self.status
    }

    /// Get stats
    pub fn stats(&self) -> &GazetteStats {
        &self.stats
    }

    /// Notice count
    pub fn notice_count(&self) -> usize {
        self.notices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gazette_new() {
        let g = SettingsGazette::new(GazetteConfig::default());
        assert_eq!(g.notice_count(), 0);
    }

    #[test]
    fn test_gazette_add_notice() {
        let mut g = SettingsGazette::new(GazetteConfig::default());
        g.add_notice(GazetteNotice::new("n1", "Notice 1", "Content"));
        assert_eq!(g.notice_count(), 1);
    }

    #[test]
    fn test_gazette_review() {
        let mut g = SettingsGazette::new(GazetteConfig::default());
        g.review();
        assert_eq!(g.status(), GazetteStatus::Review);
    }

    #[test]
    fn test_gazette_publish() {
        let mut g = SettingsGazette::new(GazetteConfig::default());
        g.publish();
        assert_eq!(g.status(), GazetteStatus::Published);
    }
}
