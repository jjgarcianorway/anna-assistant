// v0.0.717: Settings Circular - Main Circular (Phase 293)
// Main circular system

use super::config::CircularConfig;
use super::notice::{CircularNotice, CircularAttachment};
use super::stats::CircularStats;

/// Settings circular
#[derive(Debug, Clone, Default)]
pub struct SettingsCircular {
    /// Config
    config: CircularConfig,
    /// Notices
    notices: Vec<CircularNotice>,
    /// Attachments
    attachments: Vec<CircularAttachment>,
    /// Stats
    stats: CircularStats,
}

impl SettingsCircular {
    /// Create new circular system
    pub fn new(config: CircularConfig) -> Self {
        Self {
            config,
            notices: Vec::new(),
            attachments: Vec::new(),
            stats: CircularStats::default(),
        }
    }

    /// Add notice
    pub fn add_notice(&mut self, notice: CircularNotice) -> bool {
        if self.notices.len() >= self.config.max_circulars {
            return false;
        }
        self.notices.push(notice);
        self.update_stats();
        true
    }

    /// Get notice
    pub fn get_notice(&self, id: &str) -> Option<&CircularNotice> {
        self.notices.iter().find(|n| n.id == id)
    }

    /// Get notice mut
    pub fn get_notice_mut(&mut self, id: &str) -> Option<&mut CircularNotice> {
        self.notices.iter_mut().find(|n| n.id == id)
    }

    /// Add attachment
    pub fn add_attachment(&mut self, attachment: CircularAttachment) {
        self.attachments.push(attachment);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.notices, self.config.circular_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CircularStats {
        &self.stats
    }

    /// Notice count
    pub fn notice_count(&self) -> usize {
        self.notices.len()
    }
}
