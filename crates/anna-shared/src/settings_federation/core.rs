// v0.0.737: Settings Federation (Phase 313)
// Federal union for settings governance - Core

use super::types::{FederationArticle, FederationConfig, FederationStats, FederationState};

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
