// v0.0.706: Settings Bulletin - Config (Phase 282)
// Bulletin configuration

use serde::{Deserialize, Serialize};
use super::types::BulletinType;

/// Bulletin config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletinConfig {
    /// Name
    pub name: String,
    /// Bulletin type
    pub bulletin_type: BulletinType,
    /// Max posts
    pub max_posts: usize,
    /// Auto expire days
    pub auto_expire_days: usize,
}

impl BulletinConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bulletin_type: BulletinType::News,
            max_posts: 100,
            auto_expire_days: 30,
        }
    }

    /// Set type
    pub fn bulletin_type(mut self, bt: BulletinType) -> Self {
        self.bulletin_type = bt;
        self
    }

    /// Set max posts
    pub fn max_posts(mut self, max: usize) -> Self {
        self.max_posts = max;
        self
    }

    /// Set auto expire
    pub fn auto_expire_days(mut self, days: usize) -> Self {
        self.auto_expire_days = days;
        self
    }
}

impl Default for BulletinConfig {
    fn default() -> Self {
        Self::new("default")
    }
}