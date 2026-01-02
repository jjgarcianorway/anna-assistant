// v0.0.733: Settings Convention Core (Phase 309)
// Main settings convention implementation

use super::config::ConventionConfig;
use super::article::ConventionArticle;
use super::party::ConventionParty;
use super::stats::ConventionStats;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
