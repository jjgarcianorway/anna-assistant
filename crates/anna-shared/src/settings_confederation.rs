// v0.0.738: Settings Confederation (Phase 314)
// Loose union for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Confederation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConfederationType {
    /// Sovereign confederation
    #[default]
    Sovereign,
    /// Economic confederation
    Economic,
    /// Military confederation
    Military,
    /// Political confederation
    Political,
}

impl std::fmt::Display for ConfederationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sovereign => write!(f, "sovereign"),
            Self::Economic => write!(f, "economic"),
            Self::Military => write!(f, "military"),
            Self::Political => write!(f, "political"),
        }
    }
}

/// Confederation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConfederationStatus {
    /// Forming status
    #[default]
    Forming,
    /// Functional status
    Functional,
    /// Strained status
    Strained,
    /// Dissolved status
    Dissolved,
}

impl std::fmt::Display for ConfederationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forming => write!(f, "forming"),
            Self::Functional => write!(f, "functional"),
            Self::Strained => write!(f, "strained"),
            Self::Dissolved => write!(f, "dissolved"),
        }
    }
}

/// Confederation config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfederationConfig {
    /// Name
    pub name: String,
    /// Confederation type
    pub confederation_type: ConfederationType,
    /// Status
    pub status: ConfederationStatus,
    /// Max articles
    pub max_articles: usize,
}

impl ConfederationConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            confederation_type: ConfederationType::Sovereign,
            status: ConfederationStatus::Forming,
            max_articles: 100,
        }
    }

    /// Set type
    pub fn confederation_type(mut self, ct: ConfederationType) -> Self {
        self.confederation_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: ConfederationStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max articles
    pub fn max_articles(mut self, max: usize) -> Self {
        self.max_articles = max;
        self
    }
}

impl Default for ConfederationConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Confederation article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfederationArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Clause number
    pub clause: u32,
    /// Voluntary
    pub voluntary: bool,
}

impl ConfederationArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            clause: 0,
            voluntary: true,
        }
    }

    /// Set clause
    pub fn clause(mut self, c: u32) -> Self {
        self.clause = c;
        self
    }

    /// Make voluntary
    pub fn make_voluntary(&mut self) {
        self.voluntary = true;
    }

    /// Make mandatory
    pub fn make_mandatory(&mut self) {
        self.voluntary = false;
    }
}

/// Confederation member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfederationMember {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Article ID
    pub article_id: String,
}

impl ConfederationMember {
    /// Create new member
    pub fn new(key: impl Into<String>, name: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            article_id: article_id.into(),
        }
    }
}

/// Confederation stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfederationStats {
    /// Total articles
    pub total_articles: usize,
    /// Voluntary articles
    pub voluntary: usize,
    /// Functional count
    pub functional_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ConfederationStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[ConfederationArticle], confederation_type: ConfederationType) {
        self.total_articles = articles.len();
        self.voluntary = articles.iter().filter(|a| a.voluntary).count();
        *self.by_type.entry(confederation_type.to_string()).or_insert(0) += 1;
    }

    /// Voluntary rate
    pub fn voluntary_rate(&self) -> f64 {
        if self.total_articles == 0 { 0.0 } else { self.voluntary as f64 / self.total_articles as f64 * 100.0 }
    }
}

/// Settings confederation
#[derive(Debug, Clone, Default)]
pub struct SettingsConfederation {
    /// Config
    config: ConfederationConfig,
    /// Articles
    articles: Vec<ConfederationArticle>,
    /// Members
    members: Vec<ConfederationMember>,
    /// Stats
    stats: ConfederationStats,
}

impl SettingsConfederation {
    /// Create new confederation system
    pub fn new(config: ConfederationConfig) -> Self {
        Self {
            config,
            articles: Vec::new(),
            members: Vec::new(),
            stats: ConfederationStats::default(),
        }
    }

    /// Add article
    pub fn add_article(&mut self, article: ConfederationArticle) -> bool {
        if self.articles.len() >= self.config.max_articles {
            return false;
        }
        self.articles.push(article);
        self.update_stats();
        true
    }

    /// Get article
    pub fn get_article(&self, id: &str) -> Option<&ConfederationArticle> {
        self.articles.iter().find(|a| a.id == id)
    }

    /// Get article mut
    pub fn get_article_mut(&mut self, id: &str) -> Option<&mut ConfederationArticle> {
        self.articles.iter_mut().find(|a| a.id == id)
    }

    /// Add member
    pub fn add_member(&mut self, member: ConfederationMember) {
        self.members.push(member);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.articles, self.config.confederation_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ConfederationStats {
        &self.stats
    }

    /// Article count
    pub fn article_count(&self) -> usize {
        self.articles.len()
    }
}

/// Confederation registry
#[derive(Debug, Clone, Default)]
pub struct ConfederationRegistry {
    /// Confederations by ID
    confederations: HashMap<String, SettingsConfederation>,
}

impl ConfederationRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register confederation
    pub fn register(&mut self, id: impl Into<String>, confederation: SettingsConfederation) {
        self.confederations.insert(id.into(), confederation);
    }

    /// Unregister confederation
    pub fn unregister(&mut self, id: &str) -> bool {
        self.confederations.remove(id).is_some()
    }

    /// Get confederation
    pub fn get(&self, id: &str) -> Option<&SettingsConfederation> {
        self.confederations.get(id)
    }

    /// Get confederation mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConfederation> {
        self.confederations.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.confederations.len()
    }
}

/// Format confederation registry
pub fn format_confederation_registry(registry: &ConfederationRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Confederation Registry:\n");
    output.push_str(&format!("  Confederations: {}\n", registry.count()));
    output
}

/// Check if query is about confederation
pub fn is_confederation_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings confederation") || lower.contains("confederation settings") || lower.contains("loose union")
}

/// Fun fact about confederation
pub fn confederation_fun_fact() -> &'static str {
    "Anna's settings confederation establishes loose governance unions!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confederation_type_display() {
        assert_eq!(format!("{}", ConfederationType::Sovereign), "sovereign");
        assert_eq!(format!("{}", ConfederationType::Economic), "economic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ConfederationStatus::Forming), "forming");
        assert_eq!(format!("{}", ConfederationStatus::Functional), "functional");
    }

    #[test]
    fn test_config_new() {
        let c = ConfederationConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ConfederationConfig::new("test")
            .confederation_type(ConfederationType::Economic)
            .status(ConfederationStatus::Functional);
        assert_eq!(c.confederation_type, ConfederationType::Economic);
        assert_eq!(c.status, ConfederationStatus::Functional);
    }

    #[test]
    fn test_article_new() {
        let a = ConfederationArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = ConfederationArticle::new("a1", "Title", "Content")
            .clause(1);
        assert_eq!(a.clause, 1);
    }

    #[test]
    fn test_article_voluntary() {
        let mut a = ConfederationArticle::new("a1", "Title", "Content");
        a.make_mandatory();
        assert!(!a.voluntary);
        a.make_voluntary();
        assert!(a.voluntary);
    }

    #[test]
    fn test_member_new() {
        let m = ConfederationMember::new("key", "name", "a1");
        assert_eq!(m.article_id, "a1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ConfederationStats::default();
        let article = ConfederationArticle::new("a1", "Title", "Content");
        s.update(&[article], ConfederationType::Sovereign);
        assert_eq!(s.total_articles, 1);
        assert_eq!(s.voluntary, 1);
    }

    #[test]
    fn test_confederation_new() {
        let c = SettingsConfederation::new(ConfederationConfig::default());
        assert_eq!(c.article_count(), 0);
    }

    #[test]
    fn test_confederation_add_article() {
        let mut c = SettingsConfederation::new(ConfederationConfig::default());
        c.add_article(ConfederationArticle::new("a1", "Title", "Content"));
        assert_eq!(c.article_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ConfederationRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ConfederationRegistry::new();
        r.register("c1", SettingsConfederation::new(ConfederationConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_confederation_query() {
        assert!(is_confederation_query("settings confederation"));
        assert!(!is_confederation_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = confederation_fun_fact();
        assert!(fact.contains("confederation"));
    }
}
