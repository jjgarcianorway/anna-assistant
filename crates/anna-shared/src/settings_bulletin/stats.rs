// v0.0.706: Settings Bulletin - Stats (Phase 282)
// Bulletin statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::BulletinType;
use super::post::BulletinPost;

/// Bulletin stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BulletinStats {
    /// Total posts
    pub total_posts: usize,
    /// Pinned posts
    pub pinned_posts: usize,
    /// High priority posts
    pub high_priority: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BulletinStats {
    /// Update from bulletin
    pub fn update(&mut self, posts: &[BulletinPost], bulletin_type: BulletinType) {
        self.total_posts = posts.len();
        self.pinned_posts = posts.iter().filter(|p| p.pinned).count();
        self.high_priority = posts.iter().filter(|p| p.is_high_priority()).count();
        *self.by_type.entry(bulletin_type.to_string()).or_insert(0) += 1;
    }

    /// Pinned rate
    pub fn pinned_rate(&self) -> f64 {
        if self.total_posts == 0 { 0.0 } else { self.pinned_posts as f64 / self.total_posts as f64 * 100.0 }
    }
}
