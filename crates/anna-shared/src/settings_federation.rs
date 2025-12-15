// v0.0.737: Settings Federation (Phase 313)
// Federal union for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Federation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FederationType {
    /// Symmetric federation
    #[default]
    Symmetric,
    /// Asymmetric federation
    Asymmetric,
    /// Cooperative federation
    Cooperative,
    /// Dual federation
    Dual,
}

impl std::fmt::Display for FederationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Symmetric => write!(f, "symmetric"),
            Self::Asymmetric => write!(f, "asymmetric"),
            Self::Cooperative => write!(f, "cooperative"),
            Self::Dual => write!(f, "dual"),
        }
    }
}

/// Federation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FederationStatus {
    /// Constituting status
    #[default]
    Constituting,
    /// Established status
    Established,
    /// Reforming status
    Reforming,
    /// Dissolving status
    Dissolving,
}

impl std::fmt::Display for FederationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constituting => write!(f, "constituting"),
            Self::Established => write!(f, "established"),
            Self::Reforming => write!(f, "reforming"),
            Self::Dissolving => write!(f, "dissolving"),
        }
    }
}

/// Federation config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Name
    pub name: String,
    /// Federation type
    pub federation_type: FederationType,
    /// Status
    pub status: FederationStatus,
    /// Max articles
    pub max_articles: usize,
}

impl FederationConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            federation_type: FederationType::Symmetric,
            status: FederationStatus::Constituting,
            max_articles: 100,
        }
    }

    /// Set type
    pub fn federation_type(mut self, ft: FederationType) -> Self {
        self.federation_type = ft;
        self
    }

    /// Set status
    pub fn status(mut self, s: FederationStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max articles
    pub fn max_articles(mut self, max: usize) -> Self {
        self.max_articles = max;
        self
    }
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Federation article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Constitutional
    pub constitutional: bool,
}

impl FederationArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            constitutional: false,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make constitutional
    pub fn make_constitutional(&mut self) {
        self.constitutional = true;
    }

    /// Make statutory
    pub fn make_statutory(&mut self) {
        self.constitutional = false;
    }
}

/// Federation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationState {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Article ID
    pub article_id: String,
}

impl FederationState {
    /// Create new state
    pub fn new(key: impl Into<String>, name: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            article_id: article_id.into(),
        }
    }
}

/// Federation stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FederationStats {
    /// Total articles
    pub total_articles: usize,
    /// Constitutional articles
    pub constitutional: usize,
    /// Established count
    pub established_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl FederationStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[FederationArticle], federation_type: FederationType) {
        self.total_articles = articles.len();
        self.constitutional = articles.iter().filter(|a| a.constitutional).count();
        *self.by_type.entry(federation_type.to_string()).or_insert(0) += 1;
    }

    /// Constitutional rate
    pub fn constitutional_rate(&self) -> f64 {
        if self.total_articles == 0 { 0.0 } else { self.constitutional as f64 / self.total_articles as f64 * 100.0 }
    }
}

/// Settings federation
#[derive(Debug, Clone, Default)]
pub struct SettingsFederation {
    /// Config
    config: FederationConfig,
    /// Articles
    articles: Vec<FederationArticle>,
    /// States
    states: Vec<FederationState>,
    /// Stats
    stats: FederationStats,
}

impl SettingsFederation {
    /// Create new federation system
    pub fn new(config: FederationConfig) -> Self {
        Self {
            config,
            articles: Vec::new(),
            states: Vec::new(),
            stats: FederationStats::default(),
        }
    }

    /// Add article
    pub fn add_article(&mut self, article: FederationArticle) -> bool {
        if self.articles.len() >= self.config.max_articles {
            return false;
        }
        self.articles.push(article);
        self.update_stats();
        true
    }

    /// Get article
    pub fn get_article(&self, id: &str) -> Option<&FederationArticle> {
        self.articles.iter().find(|a| a.id == id)
    }

    /// Get article mut
    pub fn get_article_mut(&mut self, id: &str) -> Option<&mut FederationArticle> {
        self.articles.iter_mut().find(|a| a.id == id)
    }

    /// Add state
    pub fn add_state(&mut self, state: FederationState) {
        self.states.push(state);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.articles, self.config.federation_type);
    }

    /// Get stats
    pub fn stats(&self) -> &FederationStats {
        &self.stats
    }

    /// Article count
    pub fn article_count(&self) -> usize {
        self.articles.len()
    }
}

/// Federation registry
#[derive(Debug, Clone, Default)]
pub struct FederationRegistry {
    /// Federations by ID
    federations: HashMap<String, SettingsFederation>,
}

impl FederationRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register federation
    pub fn register(&mut self, id: impl Into<String>, federation: SettingsFederation) {
        self.federations.insert(id.into(), federation);
    }

    /// Unregister federation
    pub fn unregister(&mut self, id: &str) -> bool {
        self.federations.remove(id).is_some()
    }

    /// Get federation
    pub fn get(&self, id: &str) -> Option<&SettingsFederation> {
        self.federations.get(id)
    }

    /// Get federation mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFederation> {
        self.federations.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.federations.len()
    }
}

/// Format federation registry
pub fn format_federation_registry(registry: &FederationRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Federation Registry:\n");
    output.push_str(&format!("  Federations: {}\n", registry.count()));
    output
}

/// Check if query is about federation
pub fn is_federation_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings federation") || lower.contains("federation settings") || lower.contains("federal union")
}

/// Fun fact about federation
pub fn federation_fun_fact() -> &'static str {
    "Anna's settings federation establishes federal governance structures!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federation_type_display() {
        assert_eq!(format!("{}", FederationType::Symmetric), "symmetric");
        assert_eq!(format!("{}", FederationType::Asymmetric), "asymmetric");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", FederationStatus::Constituting), "constituting");
        assert_eq!(format!("{}", FederationStatus::Established), "established");
    }

    #[test]
    fn test_config_new() {
        let c = FederationConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = FederationConfig::new("test")
            .federation_type(FederationType::Asymmetric)
            .status(FederationStatus::Established);
        assert_eq!(c.federation_type, FederationType::Asymmetric);
        assert_eq!(c.status, FederationStatus::Established);
    }

    #[test]
    fn test_article_new() {
        let a = FederationArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = FederationArticle::new("a1", "Title", "Content")
            .section(1);
        assert_eq!(a.section, 1);
    }

    #[test]
    fn test_article_constitutional() {
        let mut a = FederationArticle::new("a1", "Title", "Content");
        a.make_constitutional();
        assert!(a.constitutional);
        a.make_statutory();
        assert!(!a.constitutional);
    }

    #[test]
    fn test_state_new() {
        let s = FederationState::new("key", "name", "a1");
        assert_eq!(s.article_id, "a1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = FederationStats::default();
        let mut article = FederationArticle::new("a1", "Title", "Content");
        article.make_constitutional();
        s.update(&[article], FederationType::Symmetric);
        assert_eq!(s.total_articles, 1);
        assert_eq!(s.constitutional, 1);
    }

    #[test]
    fn test_federation_new() {
        let f = SettingsFederation::new(FederationConfig::default());
        assert_eq!(f.article_count(), 0);
    }

    #[test]
    fn test_federation_add_article() {
        let mut f = SettingsFederation::new(FederationConfig::default());
        f.add_article(FederationArticle::new("a1", "Title", "Content"));
        assert_eq!(f.article_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = FederationRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FederationRegistry::new();
        r.register("f1", SettingsFederation::new(FederationConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_federation_query() {
        assert!(is_federation_query("settings federation"));
        assert!(!is_federation_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = federation_fun_fact();
        assert!(fact.contains("federation"));
    }
}
