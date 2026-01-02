// v0.0.732: Settings Concordat Core (Phase 308)
// Core concordat structures and implementations

use super::types::{ConcordatArticle, ConcordatConfig, ConcordatSignatory, ConcordatStats};
use std::collections::HashMap;

/// Settings concordat
#[derive(Debug, Clone, Default)]
pub struct SettingsConcordat {
    /// Config
    config: ConcordatConfig,
    /// Articles
    articles: Vec<ConcordatArticle>,
    /// Signatories
    signatories: Vec<ConcordatSignatory>,
    /// Stats
    stats: ConcordatStats,
}

impl SettingsConcordat {
    /// Create new concordat system
    pub fn new(config: ConcordatConfig) -> Self {
        Self {
            config,
            articles: Vec::new(),
            signatories: Vec::new(),
            stats: ConcordatStats::default(),
        }
    }

    /// Add article
    pub fn add_article(&mut self, article: ConcordatArticle) -> bool {
        if self.articles.len() >= self.config.max_articles {
            return false;
        }
        self.articles.push(article);
        self.update_stats();
        true
    }

    /// Get article
    pub fn get_article(&self, id: &str) -> Option<&ConcordatArticle> {
        self.articles.iter().find(|a| a.id == id)
    }

    /// Get article mut
    pub fn get_article_mut(&mut self, id: &str) -> Option<&mut ConcordatArticle> {
        self.articles.iter_mut().find(|a| a.id == id)
    }

    /// Add signatory
    pub fn add_signatory(&mut self, signatory: ConcordatSignatory) {
        self.signatories.push(signatory);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.articles, self.config.concordat_type);
    }

    /// Get stats
    pub fn stats(&self) -> &ConcordatStats {
        &self.stats
    }

    /// Article count
    pub fn article_count(&self) -> usize {
        self.articles.len()
    }
}

/// Concordat registry
#[derive(Debug, Clone, Default)]
pub struct ConcordatRegistry {
    /// Concordats by ID
    concordats: HashMap<String, SettingsConcordat>,
}

impl ConcordatRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register concordat
    pub fn register(&mut self, id: impl Into<String>, concordat: SettingsConcordat) {
        self.concordats.insert(id.into(), concordat);
    }

    /// Unregister concordat
    pub fn unregister(&mut self, id: &str) -> bool {
        self.concordats.remove(id).is_some()
    }

    /// Get concordat
    pub fn get(&self, id: &str) -> Option<&SettingsConcordat> {
        self.concordats.get(id)
    }

    /// Get concordat mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConcordat> {
        self.concordats.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.concordats.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_concordat::types::ConcordatConfig;

    #[test]
    fn test_concordat_new() {
        let c = SettingsConcordat::new(ConcordatConfig::default());
        assert_eq!(c.article_count(), 0);
    }

    #[test]
    fn test_concordat_add_article() {
        use crate::settings_concordat::types::ConcordatArticle;
        let mut c = SettingsConcordat::new(ConcordatConfig::default());
        c.add_article(ConcordatArticle::new("a1", "Title", "Content"));
        assert_eq!(c.article_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ConcordatRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ConcordatRegistry::new();
        r.register("c1", SettingsConcordat::new(ConcordatConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
