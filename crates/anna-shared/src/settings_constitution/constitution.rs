// v0.0.725: Settings Constitution (Phase 301)

use super::article::{ConstitutionArticle, ConstitutionClause};
use super::config::ConstitutionConfig;
use super::stats::ConstitutionStats;

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
