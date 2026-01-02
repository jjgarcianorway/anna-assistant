// v0.0.738: Settings Confederation
// Main confederation structure

use super::config::ConfederationConfig;
use super::article::ConfederationArticle;
use super::member::ConfederationMember;
use super::stats::ConfederationStats;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
