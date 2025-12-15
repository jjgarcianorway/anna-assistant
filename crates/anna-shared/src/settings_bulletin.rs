// v0.0.706: Settings Bulletin (Phase 282)
// Bulletin board for settings updates

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bulletin type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BulletinType {
    /// News bulletin
    #[default]
    News,
    /// Alert bulletin
    Alert,
    /// Update bulletin
    Update,
    /// Archive bulletin
    Archive,
}

impl std::fmt::Display for BulletinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::News => write!(f, "news"),
            Self::Alert => write!(f, "alert"),
            Self::Update => write!(f, "update"),
            Self::Archive => write!(f, "archive"),
        }
    }
}

/// Bulletin priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BulletinPriority {
    /// Low priority
    #[default]
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Urgent priority
    Urgent,
}

impl std::fmt::Display for BulletinPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Urgent => write!(f, "urgent"),
        }
    }
}

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

/// Bulletin registry
#[derive(Debug, Clone, Default)]
pub struct BulletinRegistry {
    /// Bulletins by ID
    bulletins: HashMap<String, SettingsBulletin>,
}

impl BulletinRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register bulletin
    pub fn register(&mut self, id: impl Into<String>, bulletin: SettingsBulletin) {
        self.bulletins.insert(id.into(), bulletin);
    }

    /// Unregister bulletin
    pub fn unregister(&mut self, id: &str) -> bool {
        self.bulletins.remove(id).is_some()
    }

    /// Get bulletin
    pub fn get(&self, id: &str) -> Option<&SettingsBulletin> {
        self.bulletins.get(id)
    }

    /// Get bulletin mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsBulletin> {
        self.bulletins.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.bulletins.len()
    }
}

/// Format bulletin registry
pub fn format_bulletin_registry(registry: &BulletinRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Bulletin Registry:\n");
    output.push_str(&format!("  Bulletins: {}\n", registry.count()));
    output
}

/// Check if query is about bulletin
pub fn is_bulletin_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings bulletin") || lower.contains("bulletin settings") || lower.contains("settings board")
}

/// Fun fact about bulletin
pub fn bulletin_fun_fact() -> &'static str {
    "Anna's settings bulletin keeps you informed about configuration updates!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulletin_type_display() {
        assert_eq!(format!("{}", BulletinType::News), "news");
        assert_eq!(format!("{}", BulletinType::Alert), "alert");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", BulletinPriority::Normal), "normal");
        assert_eq!(format!("{}", BulletinPriority::Urgent), "urgent");
    }

    #[test]
    fn test_config_new() {
        let c = BulletinConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BulletinConfig::new("test")
            .bulletin_type(BulletinType::Alert)
            .max_posts(50);
        assert_eq!(c.bulletin_type, BulletinType::Alert);
        assert_eq!(c.max_posts, 50);
    }

    #[test]
    fn test_post_new() {
        let p = BulletinPost::new("p1", "Post 1", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_post_builder() {
        let p = BulletinPost::new("p1", "Post 1", "Content")
            .priority(BulletinPriority::High)
            .pinned(true);
        assert!(p.is_high_priority());
        assert!(p.pinned);
    }

    #[test]
    fn test_item_new() {
        let i = BulletinItem::new("key", "value", "p1");
        assert_eq!(i.post_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BulletinStats::default();
        let posts = vec![BulletinPost::new("p1", "Post", "Content").pinned(true)];
        s.update(&posts, BulletinType::News);
        assert_eq!(s.total_posts, 1);
        assert_eq!(s.pinned_posts, 1);
    }

    #[test]
    fn test_bulletin_new() {
        let b = SettingsBulletin::new(BulletinConfig::default());
        assert_eq!(b.post_count(), 0);
    }

    #[test]
    fn test_bulletin_add_post() {
        let mut b = SettingsBulletin::new(BulletinConfig::default());
        b.add_post(BulletinPost::new("p1", "Post 1", "Content"));
        assert_eq!(b.post_count(), 1);
    }

    #[test]
    fn test_bulletin_get_pinned() {
        let mut b = SettingsBulletin::new(BulletinConfig::default());
        b.add_post(BulletinPost::new("p1", "Post 1", "Content").pinned(true));
        let pinned = b.get_pinned();
        assert_eq!(pinned.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BulletinRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BulletinRegistry::new();
        r.register("b1", SettingsBulletin::new(BulletinConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_bulletin_query() {
        assert!(is_bulletin_query("settings bulletin"));
        assert!(!is_bulletin_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = bulletin_fun_fact();
        assert!(fact.contains("bulletin"));
    }
}
