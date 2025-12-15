// v0.0.733: Settings Convention (Phase 309)
// Formal convention for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Convention type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConventionType {
    /// International convention
    #[default]
    International,
    /// Constitutional convention
    Constitutional,
    /// Trade convention
    Trade,
    /// Technical convention
    Technical,
}

impl std::fmt::Display for ConventionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::International => write!(f, "international"),
            Self::Constitutional => write!(f, "constitutional"),
            Self::Trade => write!(f, "trade"),
            Self::Technical => write!(f, "technical"),
        }
    }
}

/// Convention status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConventionStatus {
    /// Draft status
    #[default]
    Draft,
    /// Adopted status
    Adopted,
    /// InForce status
    InForce,
    /// Superseded status
    Superseded,
}

impl std::fmt::Display for ConventionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Adopted => write!(f, "adopted"),
            Self::InForce => write!(f, "in_force"),
            Self::Superseded => write!(f, "superseded"),
        }
    }
}

/// Convention config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionConfig {
    /// Name
    pub name: String,
    /// Convention type
    pub convention_type: ConventionType,
    /// Status
    pub status: ConventionStatus,
    /// Max articles
    pub max_articles: usize,
}

impl ConventionConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            convention_type: ConventionType::International,
            status: ConventionStatus::Draft,
            max_articles: 100,
        }
    }

    /// Set type
    pub fn convention_type(mut self, ct: ConventionType) -> Self {
        self.convention_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: ConventionStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max articles
    pub fn max_articles(mut self, max: usize) -> Self {
        self.max_articles = max;
        self
    }
}

impl Default for ConventionConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Convention article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub number: u32,
    /// Binding
    pub binding: bool,
}

impl ConventionArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            number: 0,
            binding: true,
        }
    }

    /// Set number
    pub fn number(mut self, n: u32) -> Self {
        self.number = n;
        self
    }

    /// Make binding
    pub fn make_binding(&mut self) {
        self.binding = true;
    }

    /// Make advisory
    pub fn make_advisory(&mut self) {
        self.binding = false;
    }
}

/// Convention party
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionParty {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Article ID
    pub article_id: String,
}

impl ConventionParty {
    /// Create new party
    pub fn new(key: impl Into<String>, name: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            article_id: article_id.into(),
        }
    }
}

/// Convention stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConventionStats {
    /// Total articles
    pub total_articles: usize,
    /// Binding articles
    pub binding: usize,
    /// In force count
    pub in_force_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ConventionStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[ConventionArticle], convention_type: ConventionType) {
        self.total_articles = articles.len();
        self.binding = articles.iter().filter(|a| a.binding).count();
        *self.by_type.entry(convention_type.to_string()).or_insert(0) += 1;
    }

    /// Binding rate
    pub fn binding_rate(&self) -> f64 {
        if self.total_articles == 0 { 0.0 } else { self.binding as f64 / self.total_articles as f64 * 100.0 }
    }
}

/// Settings convention
#[derive(Debug, Clone, Default)]
pub struct SettingsConvention {
    /// Config
    config: ConventionConfig,
    /// Articles
    articles: Vec<ConventionArticle>,
    /// Parties
    parties: Vec<ConventionParty>,
    /// Stats
    stats: ConventionStats,
}

impl SettingsConvention {
    /// Create new convention system
    pub fn new(config: ConventionConfig) -> Self {
        Self {
            config,
            articles: Vec::new(),
            parties: Vec::new(),
            stats: ConventionStats::default(),
        }
    }

    /// Add article
    pub fn add_article(&mut self, article: ConventionArticle) -> bool {
        if self.articles.len() >= self.config.max_articles {
            return false;
        }
        self.articles.push(article);
        self.update_stats();
        true
    }

    /// Get article
    pub fn get_article(&self, id: &str) -> Option<&ConventionArticle> {
        self.articles.iter().find(|a| a.id == id)
    }

    /// Get article mut
    pub fn get_article_mut(&mut self, id: &str) -> Option<&mut ConventionArticle> {
        self.articles.iter_mut().find(|a| a.id == id)
    }

    /// Add party
    pub fn add_party(&mut self, party: ConventionParty) {
        self.parties.push(party);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.articles, self.config.convention_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ConventionStats {
        &self.stats
    }

    /// Article count
    pub fn article_count(&self) -> usize {
        self.articles.len()
    }
}

/// Convention registry
#[derive(Debug, Clone, Default)]
pub struct ConventionRegistry {
    /// Conventions by ID
    conventions: HashMap<String, SettingsConvention>,
}

impl ConventionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register convention
    pub fn register(&mut self, id: impl Into<String>, convention: SettingsConvention) {
        self.conventions.insert(id.into(), convention);
    }

    /// Unregister convention
    pub fn unregister(&mut self, id: &str) -> bool {
        self.conventions.remove(id).is_some()
    }

    /// Get convention
    pub fn get(&self, id: &str) -> Option<&SettingsConvention> {
        self.conventions.get(id)
    }

    /// Get convention mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConvention> {
        self.conventions.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.conventions.len()
    }
}

/// Format convention registry
pub fn format_convention_registry(registry: &ConventionRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Convention Registry:\n");
    output.push_str(&format!("  Conventions: {}\n", registry.count()));
    output
}

/// Check if query is about convention
pub fn is_convention_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings convention") || lower.contains("convention settings") || lower.contains("formal gathering")
}

/// Fun fact about convention
pub fn convention_fun_fact() -> &'static str {
    "Anna's settings convention establishes formal governance standards!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convention_type_display() {
        assert_eq!(format!("{}", ConventionType::International), "international");
        assert_eq!(format!("{}", ConventionType::Constitutional), "constitutional");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ConventionStatus::Draft), "draft");
        assert_eq!(format!("{}", ConventionStatus::InForce), "in_force");
    }

    #[test]
    fn test_config_new() {
        let c = ConventionConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ConventionConfig::new("test")
            .convention_type(ConventionType::Constitutional)
            .status(ConventionStatus::Adopted);
        assert_eq!(c.convention_type, ConventionType::Constitutional);
        assert_eq!(c.status, ConventionStatus::Adopted);
    }

    #[test]
    fn test_article_new() {
        let a = ConventionArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = ConventionArticle::new("a1", "Title", "Content")
            .number(1);
        assert_eq!(a.number, 1);
    }

    #[test]
    fn test_article_binding() {
        let mut a = ConventionArticle::new("a1", "Title", "Content");
        a.make_advisory();
        assert!(!a.binding);
        a.make_binding();
        assert!(a.binding);
    }

    #[test]
    fn test_party_new() {
        let p = ConventionParty::new("key", "name", "a1");
        assert_eq!(p.article_id, "a1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ConventionStats::default();
        let article = ConventionArticle::new("a1", "Title", "Content");
        s.update(&[article], ConventionType::International);
        assert_eq!(s.total_articles, 1);
        assert_eq!(s.binding, 1);
    }

    #[test]
    fn test_convention_new() {
        let c = SettingsConvention::new(ConventionConfig::default());
        assert_eq!(c.article_count(), 0);
    }

    #[test]
    fn test_convention_add_article() {
        let mut c = SettingsConvention::new(ConventionConfig::default());
        c.add_article(ConventionArticle::new("a1", "Title", "Content"));
        assert_eq!(c.article_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ConventionRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ConventionRegistry::new();
        r.register("c1", SettingsConvention::new(ConventionConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_convention_query() {
        assert!(is_convention_query("settings convention"));
        assert!(!is_convention_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = convention_fun_fact();
        assert!(fact.contains("convention"));
    }
}
