// v0.0.723: Settings Statute (Phase 299)
// Written laws for settings governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statute type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StatuteType {
    /// General statute
    #[default]
    General,
    /// Criminal statute
    Criminal,
    /// Civil statute
    Civil,
    /// Administrative statute
    Administrative,
}

impl std::fmt::Display for StatuteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General => write!(f, "general"),
            Self::Criminal => write!(f, "criminal"),
            Self::Civil => write!(f, "civil"),
            Self::Administrative => write!(f, "administrative"),
        }
    }
}

/// Statute scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StatuteScope {
    /// Federal scope
    #[default]
    Federal,
    /// State scope
    State,
    /// Local scope
    Local,
    /// International scope
    International,
}

impl std::fmt::Display for StatuteScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Federal => write!(f, "federal"),
            Self::State => write!(f, "state"),
            Self::Local => write!(f, "local"),
            Self::International => write!(f, "international"),
        }
    }
}

/// Statute config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatuteConfig {
    /// Name
    pub name: String,
    /// Statute type
    pub statute_type: StatuteType,
    /// Scope
    pub scope: StatuteScope,
    /// Max statutes
    pub max_statutes: usize,
}

impl StatuteConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            statute_type: StatuteType::General,
            scope: StatuteScope::Federal,
            max_statutes: 200,
        }
    }

    /// Set type
    pub fn statute_type(mut self, st: StatuteType) -> Self {
        self.statute_type = st;
        self
    }

    /// Set scope
    pub fn scope(mut self, s: StatuteScope) -> Self {
        self.scope = s;
        self
    }

    /// Set max statutes
    pub fn max_statutes(mut self, max: usize) -> Self {
        self.max_statutes = max;
        self
    }
}

impl Default for StatuteConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Statute article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatuteArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub number: String,
    /// Enacted
    pub enacted: bool,
}

impl StatuteArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            number: String::new(),
            enacted: false,
        }
    }

    /// Set number
    pub fn number(mut self, n: impl Into<String>) -> Self {
        self.number = n.into();
        self
    }

    /// Enact article
    pub fn enact(&mut self) {
        self.enacted = true;
    }

    /// Repeal article
    pub fn repeal(&mut self) {
        self.enacted = false;
    }
}

/// Statute subsection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatuteSubsection {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Article ID
    pub article_id: String,
}

impl StatuteSubsection {
    /// Create new subsection
    pub fn new(key: impl Into<String>, value: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            article_id: article_id.into(),
        }
    }
}

/// Statute stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatuteStats {
    /// Total statutes
    pub total_statutes: usize,
    /// Enacted statutes
    pub enacted: usize,
    /// Federal count
    pub federal_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl StatuteStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[StatuteArticle], statute_type: StatuteType) {
        self.total_statutes = articles.len();
        self.enacted = articles.iter().filter(|a| a.enacted).count();
        *self.by_type.entry(statute_type.to_string()).or_insert(0) += 1;
    }

    /// Enacted rate
    pub fn enacted_rate(&self) -> f64 {
        if self.total_statutes == 0 { 0.0 } else { self.enacted as f64 / self.total_statutes as f64 * 100.0 }
    }
}

/// Settings statute
#[derive(Debug, Clone, Default)]
pub struct SettingsStatute {
    /// Config
    config: StatuteConfig,
    /// Articles
    articles: Vec<StatuteArticle>,
    /// Subsections
    subsections: Vec<StatuteSubsection>,
    /// Stats
    stats: StatuteStats,
}

impl SettingsStatute {
    /// Create new statute system
    pub fn new(config: StatuteConfig) -> Self {
        Self {
            config,
            articles: Vec::new(),
            subsections: Vec::new(),
            stats: StatuteStats::default(),
        }
    }

    /// Add article
    pub fn add_article(&mut self, article: StatuteArticle) -> bool {
        if self.articles.len() >= self.config.max_statutes {
            return false;
        }
        self.articles.push(article);
        self.update_stats();
        true
    }

    /// Get article
    pub fn get_article(&self, id: &str) -> Option<&StatuteArticle> {
        self.articles.iter().find(|a| a.id == id)
    }

    /// Get article mut
    pub fn get_article_mut(&mut self, id: &str) -> Option<&mut StatuteArticle> {
        self.articles.iter_mut().find(|a| a.id == id)
    }

    /// Add subsection
    pub fn add_subsection(&mut self, subsection: StatuteSubsection) {
        self.subsections.push(subsection);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.articles, self.config.statute_type);
    }

    /// Get stats
    pub fn stats(&self) -> &StatuteStats {
        &self.stats
    }

    /// Article count
    pub fn article_count(&self) -> usize {
        self.articles.len()
    }
}

/// Statute registry
#[derive(Debug, Clone, Default)]
pub struct StatuteRegistry {
    /// Statutes by ID
    statutes: HashMap<String, SettingsStatute>,
}

impl StatuteRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register statute
    pub fn register(&mut self, id: impl Into<String>, statute: SettingsStatute) {
        self.statutes.insert(id.into(), statute);
    }

    /// Unregister statute
    pub fn unregister(&mut self, id: &str) -> bool {
        self.statutes.remove(id).is_some()
    }

    /// Get statute
    pub fn get(&self, id: &str) -> Option<&SettingsStatute> {
        self.statutes.get(id)
    }

    /// Get statute mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsStatute> {
        self.statutes.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.statutes.len()
    }
}

/// Format statute registry
pub fn format_statute_registry(registry: &StatuteRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Statute Registry:\n");
    output.push_str(&format!("  Statutes: {}\n", registry.count()));
    output
}

/// Check if query is about statute
pub fn is_statute_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings statute") || lower.contains("statute settings") || lower.contains("written law")
}

/// Fun fact about statute
pub fn statute_fun_fact() -> &'static str {
    "Anna's settings statute codifies configuration rules into written law!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statute_type_display() {
        assert_eq!(format!("{}", StatuteType::General), "general");
        assert_eq!(format!("{}", StatuteType::Administrative), "administrative");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", StatuteScope::Federal), "federal");
        assert_eq!(format!("{}", StatuteScope::International), "international");
    }

    #[test]
    fn test_config_new() {
        let c = StatuteConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = StatuteConfig::new("test")
            .statute_type(StatuteType::Civil)
            .scope(StatuteScope::State);
        assert_eq!(c.statute_type, StatuteType::Civil);
        assert_eq!(c.scope, StatuteScope::State);
    }

    #[test]
    fn test_article_new() {
        let a = StatuteArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = StatuteArticle::new("a1", "Title", "Content")
            .number("Article 1");
        assert_eq!(a.number, "Article 1");
    }

    #[test]
    fn test_article_enact_repeal() {
        let mut a = StatuteArticle::new("a1", "Title", "Content");
        a.enact();
        assert!(a.enacted);
        a.repeal();
        assert!(!a.enacted);
    }

    #[test]
    fn test_subsection_new() {
        let s = StatuteSubsection::new("key", "value", "a1");
        assert_eq!(s.article_id, "a1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = StatuteStats::default();
        let mut article = StatuteArticle::new("a1", "Title", "Content");
        article.enact();
        s.update(&[article], StatuteType::General);
        assert_eq!(s.total_statutes, 1);
        assert_eq!(s.enacted, 1);
    }

    #[test]
    fn test_statute_new() {
        let s = SettingsStatute::new(StatuteConfig::default());
        assert_eq!(s.article_count(), 0);
    }

    #[test]
    fn test_statute_add_article() {
        let mut s = SettingsStatute::new(StatuteConfig::default());
        s.add_article(StatuteArticle::new("a1", "Title", "Content"));
        assert_eq!(s.article_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = StatuteRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = StatuteRegistry::new();
        r.register("s1", SettingsStatute::new(StatuteConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_statute_query() {
        assert!(is_statute_query("settings statute"));
        assert!(!is_statute_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = statute_fun_fact();
        assert!(fact.contains("statute"));
    }
}
