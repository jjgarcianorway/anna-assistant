// v0.0.725: Settings Constitution (Phase 301)
// Supreme law for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Constitution type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConstitutionType {
    /// Written constitution
    #[default]
    Written,
    /// Unwritten constitution
    Unwritten,
    /// Codified constitution
    Codified,
    /// Uncodified constitution
    Uncodified,
}

impl std::fmt::Display for ConstitutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Written => write!(f, "written"),
            Self::Unwritten => write!(f, "unwritten"),
            Self::Codified => write!(f, "codified"),
            Self::Uncodified => write!(f, "uncodified"),
        }
    }
}

/// Constitution branch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConstitutionBranch {
    /// Executive branch
    #[default]
    Executive,
    /// Legislative branch
    Legislative,
    /// Judicial branch
    Judicial,
    /// Administrative branch
    Administrative,
}

impl std::fmt::Display for ConstitutionBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Executive => write!(f, "executive"),
            Self::Legislative => write!(f, "legislative"),
            Self::Judicial => write!(f, "judicial"),
            Self::Administrative => write!(f, "administrative"),
        }
    }
}

/// Constitution config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionConfig {
    /// Name
    pub name: String,
    /// Constitution type
    pub constitution_type: ConstitutionType,
    /// Branch
    pub branch: ConstitutionBranch,
    /// Max articles
    pub max_articles: usize,
}

impl ConstitutionConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            constitution_type: ConstitutionType::Written,
            branch: ConstitutionBranch::Executive,
            max_articles: 100,
        }
    }

    /// Set type
    pub fn constitution_type(mut self, ct: ConstitutionType) -> Self {
        self.constitution_type = ct;
        self
    }

    /// Set branch
    pub fn branch(mut self, b: ConstitutionBranch) -> Self {
        self.branch = b;
        self
    }

    /// Set max articles
    pub fn max_articles(mut self, max: usize) -> Self {
        self.max_articles = max;
        self
    }
}

impl Default for ConstitutionConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Constitution article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub number: u32,
    /// Ratified
    pub ratified: bool,
}

impl ConstitutionArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            number: 0,
            ratified: false,
        }
    }

    /// Set number
    pub fn number(mut self, n: u32) -> Self {
        self.number = n;
        self
    }

    /// Ratify article
    pub fn ratify(&mut self) {
        self.ratified = true;
    }

    /// Repeal article
    pub fn repeal(&mut self) {
        self.ratified = false;
    }
}

/// Constitution clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionClause {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Article ID
    pub article_id: String,
}

impl ConstitutionClause {
    /// Create new clause
    pub fn new(key: impl Into<String>, value: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            article_id: article_id.into(),
        }
    }
}

/// Constitution stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstitutionStats {
    /// Total articles
    pub total_articles: usize,
    /// Ratified articles
    pub ratified: usize,
    /// Executive count
    pub executive_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ConstitutionStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[ConstitutionArticle], constitution_type: ConstitutionType) {
        self.total_articles = articles.len();
        self.ratified = articles.iter().filter(|a| a.ratified).count();
        *self.by_type.entry(constitution_type.to_string()).or_insert(0) += 1;
    }

    /// Ratified rate
    pub fn ratified_rate(&self) -> f64 {
        if self.total_articles == 0 { 0.0 } else { self.ratified as f64 / self.total_articles as f64 * 100.0 }
    }
}

/// Settings constitution
#[derive(Debug, Clone, Default)]
pub struct SettingsConstitution {
    /// Config
    config: ConstitutionConfig,
    /// Articles
    articles: Vec<ConstitutionArticle>,
    /// Clauses
    clauses: Vec<ConstitutionClause>,
    /// Stats
    stats: ConstitutionStats,
}

impl SettingsConstitution {
    /// Create new constitution system
    pub fn new(config: ConstitutionConfig) -> Self {
        Self {
            config,
            articles: Vec::new(),
            clauses: Vec::new(),
            stats: ConstitutionStats::default(),
        }
    }

    /// Add article
    pub fn add_article(&mut self, article: ConstitutionArticle) -> bool {
        if self.articles.len() >= self.config.max_articles {
            return false;
        }
        self.articles.push(article);
        self.update_stats();
        true
    }

    /// Get article
    pub fn get_article(&self, id: &str) -> Option<&ConstitutionArticle> {
        self.articles.iter().find(|a| a.id == id)
    }

    /// Get article mut
    pub fn get_article_mut(&mut self, id: &str) -> Option<&mut ConstitutionArticle> {
        self.articles.iter_mut().find(|a| a.id == id)
    }

    /// Add clause
    pub fn add_clause(&mut self, clause: ConstitutionClause) {
        self.clauses.push(clause);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.articles, self.config.constitution_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ConstitutionStats {
        &self.stats
    }

    /// Article count
    pub fn article_count(&self) -> usize {
        self.articles.len()
    }
}

/// Constitution registry
#[derive(Debug, Clone, Default)]
pub struct ConstitutionRegistry {
    /// Constitutions by ID
    constitutions: HashMap<String, SettingsConstitution>,
}

impl ConstitutionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register constitution
    pub fn register(&mut self, id: impl Into<String>, constitution: SettingsConstitution) {
        self.constitutions.insert(id.into(), constitution);
    }

    /// Unregister constitution
    pub fn unregister(&mut self, id: &str) -> bool {
        self.constitutions.remove(id).is_some()
    }

    /// Get constitution
    pub fn get(&self, id: &str) -> Option<&SettingsConstitution> {
        self.constitutions.get(id)
    }

    /// Get constitution mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConstitution> {
        self.constitutions.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.constitutions.len()
    }
}

/// Format constitution registry
pub fn format_constitution_registry(registry: &ConstitutionRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Constitution Registry:\n");
    output.push_str(&format!("  Constitutions: {}\n", registry.count()));
    output
}

/// Check if query is about constitution
pub fn is_constitution_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings constitution") || lower.contains("constitution settings") || lower.contains("supreme law")
}

/// Fun fact about constitution
pub fn constitution_fun_fact() -> &'static str {
    "Anna's settings constitution establishes supreme governance principles!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constitution_type_display() {
        assert_eq!(format!("{}", ConstitutionType::Written), "written");
        assert_eq!(format!("{}", ConstitutionType::Codified), "codified");
    }

    #[test]
    fn test_branch_display() {
        assert_eq!(format!("{}", ConstitutionBranch::Executive), "executive");
        assert_eq!(format!("{}", ConstitutionBranch::Judicial), "judicial");
    }

    #[test]
    fn test_config_new() {
        let c = ConstitutionConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ConstitutionConfig::new("test")
            .constitution_type(ConstitutionType::Codified)
            .branch(ConstitutionBranch::Legislative);
        assert_eq!(c.constitution_type, ConstitutionType::Codified);
        assert_eq!(c.branch, ConstitutionBranch::Legislative);
    }

    #[test]
    fn test_article_new() {
        let a = ConstitutionArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = ConstitutionArticle::new("a1", "Title", "Content")
            .number(1);
        assert_eq!(a.number, 1);
    }

    #[test]
    fn test_article_ratify_repeal() {
        let mut a = ConstitutionArticle::new("a1", "Title", "Content");
        a.ratify();
        assert!(a.ratified);
        a.repeal();
        assert!(!a.ratified);
    }

    #[test]
    fn test_clause_new() {
        let c = ConstitutionClause::new("key", "value", "a1");
        assert_eq!(c.article_id, "a1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ConstitutionStats::default();
        let mut article = ConstitutionArticle::new("a1", "Title", "Content");
        article.ratify();
        s.update(&[article], ConstitutionType::Written);
        assert_eq!(s.total_articles, 1);
        assert_eq!(s.ratified, 1);
    }

    #[test]
    fn test_constitution_new() {
        let c = SettingsConstitution::new(ConstitutionConfig::default());
        assert_eq!(c.article_count(), 0);
    }

    #[test]
    fn test_constitution_add_article() {
        let mut c = SettingsConstitution::new(ConstitutionConfig::default());
        c.add_article(ConstitutionArticle::new("a1", "Title", "Content"));
        assert_eq!(c.article_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ConstitutionRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ConstitutionRegistry::new();
        r.register("c1", SettingsConstitution::new(ConstitutionConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_constitution_query() {
        assert!(is_constitution_query("settings constitution"));
        assert!(!is_constitution_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = constitution_fun_fact();
        assert!(fact.contains("constitution"));
    }
}
