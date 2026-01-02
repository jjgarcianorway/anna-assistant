// v0.0.710: Settings Brief - Brief (Phase 286)
// Main SettingsBrief implementation

use super::stats::BriefStats;
use super::types::{BriefAttachment, BriefConfig, BriefPoint};

/// Settings brief
#[derive(Debug, Clone, Default)]
pub struct SettingsBrief {
    /// Config
    config: BriefConfig,
    /// Points
    points: Vec<BriefPoint>,
    /// Attachments
    attachments: Vec<BriefAttachment>,
    /// Stats
    stats: BriefStats,
}

impl SettingsBrief {
    /// Create new brief
    pub fn new(config: BriefConfig) -> Self {
        Self {
            config,
            points: Vec::new(),
            attachments: Vec::new(),
            stats: BriefStats::default(),
        }
    }

    /// Add point
    pub fn add_point(&mut self, point: BriefPoint) -> bool {
        if self.points.len() >= self.config.max_points {
            return false;
        }
        self.points.push(point);
        self.update_stats();
        true
    }

    /// Get point
    pub fn get_point(&self, id: &str) -> Option<&BriefPoint> {
        self.points.iter().find(|p| p.id == id)
    }

    /// Get point mut
    pub fn get_point_mut(&mut self, id: &str) -> Option<&mut BriefPoint> {
        self.points.iter_mut().find(|p| p.id == id)
    }

    /// Add attachment
    pub fn add_attachment(&mut self, attachment: BriefAttachment) {
        self.attachments.push(attachment);
    }

    /// Get attachment count
    pub fn attachment_count(&self) -> usize {
        self.attachments.len()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.points, self.config.brief_type);
    }

    /// Get stats
    pub fn stats(&self) -> &BriefStats {
        &self.stats
    }

    /// Point count
    pub fn point_count(&self) -> usize {
        self.points.len()
    }
}
