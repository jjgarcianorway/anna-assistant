// v0.0.785: Settings Retreat - Retreat (Phase 361)

use super::config::RetreatConfig;
use super::visitor::RetreatVisitor;
use super::guide::RetreatGuide;
use super::stats::RetreatStats;

/// Settings retreat
#[derive(Debug, Clone, Default)]
pub struct SettingsRetreat {
    /// Config
    config: RetreatConfig,
    /// Visitors
    visitors: Vec<RetreatVisitor>,
    /// Guides
    guides: Vec<RetreatGuide>,
    /// Stats
    stats: RetreatStats,
}

impl SettingsRetreat {
    /// Create new retreat system
    pub fn new(config: RetreatConfig) -> Self {
        Self {
            config,
            visitors: Vec::new(),
            guides: Vec::new(),
            stats: RetreatStats::default(),
        }
    }

    /// Add visitor
    pub fn add_visitor(&mut self, visitor: RetreatVisitor) -> bool {
        if self.visitors.len() >= self.config.max_visitors {
            return false;
        }
        self.visitors.push(visitor);
        self.update_stats();
        true
    }

    /// Get visitor
    pub fn get_visitor(&self, id: &str) -> Option<&RetreatVisitor> {
        self.visitors.iter().find(|v| v.id == id)
    }

    /// Get visitor mut
    pub fn get_visitor_mut(&mut self, id: &str) -> Option<&mut RetreatVisitor> {
        self.visitors.iter_mut().find(|v| v.id == id)
    }

    /// Add guide
    pub fn add_guide(&mut self, guide: RetreatGuide) {
        self.guides.push(guide);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.visitors, self.config.retreat_type);
    }

    /// Get stats
    pub fn stats(&self) -> &RetreatStats {
        &self.stats
    }

    /// Visitor count
    pub fn visitor_count(&self) -> usize {
        self.visitors.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retreat_new() {
        let r = SettingsRetreat::new(RetreatConfig::default());
        assert_eq!(r.visitor_count(), 0);
    }

    #[test]
    fn test_retreat_add_visitor() {
        let mut r = SettingsRetreat::new(RetreatConfig::default());
        r.add_visitor(RetreatVisitor::new("v1", "Title", "Content"));
        assert_eq!(r.visitor_count(), 1);
    }
}
