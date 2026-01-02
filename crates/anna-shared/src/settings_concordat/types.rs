// v0.0.732: Settings Concordat Types (Phase 308)
// Type definitions for settings concordat

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Concordat type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConcordatType {
    /// Papal concordat
    #[default]
    Papal,
    /// Diplomatic concordat
    Diplomatic,
    /// Administrative concordat
    Administrative,
    /// Cultural concordat
    Cultural,
}

impl std::fmt::Display for ConcordatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Papal => write!(f, "papal"),
            Self::Diplomatic => write!(f, "diplomatic"),
            Self::Administrative => write!(f, "administrative"),
            Self::Cultural => write!(f, "cultural"),
        }
    }
}

/// Concordat status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ConcordatStatus {
    /// Negotiating status
    #[default]
    Negotiating,
    /// Signed status
    Signed,
    /// Binding status
    Binding,
    /// Rescinded status
    Rescinded,
}

impl std::fmt::Display for ConcordatStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negotiating => write!(f, "negotiating"),
            Self::Signed => write!(f, "signed"),
            Self::Binding => write!(f, "binding"),
            Self::Rescinded => write!(f, "rescinded"),
        }
    }
}

/// Concordat config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcordatConfig {
    /// Name
    pub name: String,
    /// Concordat type
    pub concordat_type: ConcordatType,
    /// Status
    pub status: ConcordatStatus,
    /// Max articles
    pub max_articles: usize,
}

impl ConcordatConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            concordat_type: ConcordatType::Papal,
            status: ConcordatStatus::Negotiating,
            max_articles: 100,
        }
    }

    /// Set type
    pub fn concordat_type(mut self, ct: ConcordatType) -> Self {
        self.concordat_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: ConcordatStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max articles
    pub fn max_articles(mut self, max: usize) -> Self {
        self.max_articles = max;
        self
    }
}

impl Default for ConcordatConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Concordat article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcordatArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Canonical
    pub canonical: bool,
}

impl ConcordatArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            canonical: false,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make canonical
    pub fn make_canonical(&mut self) {
        self.canonical = true;
    }

    /// Make non-canonical
    pub fn make_non_canonical(&mut self) {
        self.canonical = false;
    }
}

/// Concordat signatory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcordatSignatory {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Article ID
    pub article_id: String,
}

impl ConcordatSignatory {
    /// Create new signatory
    pub fn new(key: impl Into<String>, name: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            article_id: article_id.into(),
        }
    }
}

/// Concordat stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConcordatStats {
    /// Total articles
    pub total_articles: usize,
    /// Canonical articles
    pub canonical: usize,
    /// Binding count
    pub binding_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ConcordatStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[ConcordatArticle], concordat_type: ConcordatType) {
        self.total_articles = articles.len();
        self.canonical = articles.iter().filter(|a| a.canonical).count();
        *self.by_type.entry(concordat_type.to_string()).or_insert(0) += 1;
    }

    /// Canonical rate
    pub fn canonical_rate(&self) -> f64 {
        if self.total_articles == 0 { 0.0 } else { self.canonical as f64 / self.total_articles as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concordat_type_display() {
        assert_eq!(format!("{}", ConcordatType::Papal), "papal");
        assert_eq!(format!("{}", ConcordatType::Diplomatic), "diplomatic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ConcordatStatus::Negotiating), "negotiating");
        assert_eq!(format!("{}", ConcordatStatus::Binding), "binding");
    }

    #[test]
    fn test_config_new() {
        let c = ConcordatConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ConcordatConfig::new("test")
            .concordat_type(ConcordatType::Diplomatic)
            .status(ConcordatStatus::Signed);
        assert_eq!(c.concordat_type, ConcordatType::Diplomatic);
        assert_eq!(c.status, ConcordatStatus::Signed);
    }

    #[test]
    fn test_article_new() {
        let a = ConcordatArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = ConcordatArticle::new("a1", "Title", "Content")
            .section(1);
        assert_eq!(a.section, 1);
    }

    #[test]
    fn test_article_canonical() {
        let mut a = ConcordatArticle::new("a1", "Title", "Content");
        a.make_canonical();
        assert!(a.canonical);
        a.make_non_canonical();
        assert!(!a.canonical);
    }

    #[test]
    fn test_signatory_new() {
        let s = ConcordatSignatory::new("key", "name", "a1");
        assert_eq!(s.article_id, "a1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ConcordatStats::default();
        let mut article = ConcordatArticle::new("a1", "Title", "Content");
        article.make_canonical();
        s.update(&[article], ConcordatType::Papal);
        assert_eq!(s.total_articles, 1);
        assert_eq!(s.canonical, 1);
    }
}
