// v0.0.723: Settings Statute Main Implementation (Phase 299)
// Core statute system

use super::types::{StatuteArticle, StatuteConfig, StatuteStats, StatuteSubsection};

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
