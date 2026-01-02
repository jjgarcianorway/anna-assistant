// v0.0.710: Settings Brief - Stats (Phase 286)
// Statistics tracking for briefs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{BriefPoint, BriefType};

/// Brief stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BriefStats {
    /// Total points
    pub total_points: usize,
    /// Action items
    pub action_items: usize,
    /// High priority
    pub high_priority: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BriefStats {
    /// Update from points
    pub fn update(&mut self, points: &[BriefPoint], brief_type: BriefType) {
        self.total_points = points.len();
        self.action_items = points.iter().filter(|p| p.action_required).count();
        self.high_priority = points.iter().filter(|p| p.priority >= 3).count();
        *self.by_type.entry(brief_type.to_string()).or_insert(0) += 1;
    }

    /// Action rate
    pub fn action_rate(&self) -> f64 {
        if self.total_points == 0 { 0.0 } else { self.action_items as f64 / self.total_points as f64 * 100.0 }
    }
}
