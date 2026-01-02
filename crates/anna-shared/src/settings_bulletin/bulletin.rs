// v0.0.706: Settings Bulletin - Main (Phase 282)
// Main bulletin board implementation

use super::config::BulletinConfig;
use super::post::{BulletinPost, BulletinItem};
use super::stats::BulletinStats;

/// Settings bulletin
#[derive(Debug, Clone, Default)]
pub struct SettingsBulletin {
    /// Config
    config: BulletinConfig,
    /// Posts
    posts: Vec<BulletinPost>,
    /// Items
    items: Vec<BulletinItem>,
    /// Stats
    stats: BulletinStats,
}

impl SettingsBulletin {
    /// Create new bulletin
    pub fn new(config: BulletinConfig) -> Self {
        Self {
            config,
            posts: Vec::new(),
            items: Vec::new(),
            stats: BulletinStats::default(),
        }
    }

    /// Add post
    pub fn add_post(&mut self, post: BulletinPost) -> bool {
        if self.posts.len() >= self.config.max_posts {
            return false;
        }
        self.posts.push(post);
        self.update_stats();
        true
    }

    /// Get post
    pub fn get_post(&self, id: &str) -> Option<&BulletinPost> {
        self.posts.iter().find(|p| p.id == id)
    }

    /// Add item
    pub fn add_item(&mut self, item: BulletinItem) {
        self.items.push(item);
    }

    /// Get pinned posts
    pub fn get_pinned(&self) -> Vec<&BulletinPost> {
        self.posts.iter().filter(|p| p.pinned).collect()
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.posts, self.config.bulletin_type);
    }

    /// Get stats
    pub fn stats(&self) -> &BulletinStats {
        &self.stats
    }

    /// Post count
    pub fn post_count(&self) -> usize {
        self.posts.len()
    }
}
