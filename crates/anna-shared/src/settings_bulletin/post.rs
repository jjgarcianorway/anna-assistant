// v0.0.706: Settings Bulletin - Post (Phase 282)
// Bulletin posts and items

use serde::{Deserialize, Serialize};
use super::types::BulletinPriority;

/// Bulletin post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletinPost {
    /// Post ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Priority
    pub priority: BulletinPriority,
    /// Posted date
    pub posted_date: String,
    /// Pinned
    pub pinned: bool,
}

impl BulletinPost {
    /// Create new post
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            priority: BulletinPriority::Normal,
            posted_date: String::new(),
            pinned: false,
        }
    }

    /// Set priority
    pub fn priority(mut self, p: BulletinPriority) -> Self {
        self.priority = p;
        self
    }

    /// Set posted date
    pub fn posted_date(mut self, date: impl Into<String>) -> Self {
        self.posted_date = date.into();
        self
    }

    /// Set pinned
    pub fn pinned(mut self, pin: bool) -> Self {
        self.pinned = pin;
        self
    }

    /// Is high priority
    pub fn is_high_priority(&self) -> bool {
        matches!(self.priority, BulletinPriority::High | BulletinPriority::Urgent)
    }
}

/// Bulletin item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletinItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Post ID
    pub post_id: String,
}

impl BulletinItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, post_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            post_id: post_id.into(),
        }
    }
}