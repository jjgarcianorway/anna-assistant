// v0.0.733: Settings Convention Stats (Phase 309)
// Convention statistics implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::article::ConventionArticle;
use super::types::ConventionType;

/// Convention stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConventionStats {
    /// Total articles
    pub total_articles: usize,
    /// Binding articles
    pub binding: usize,
    /// In force count
    pub in_force_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ConventionStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[ConventionArticle], convention_type: ConventionType) {
        self.total_articles = articles.len();
        self.binding = articles.iter().filter(|a| a.binding).count();
        *self.by_type.entry(convention_type.to_string()).or_insert(0) += 1;
    }

    /// Binding rate
    pub fn binding_rate(&self) -> f64 {
        if self.total_articles == 0 { 0.0 } else { self.binding as f64 / self.total_articles as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = ConventionStats::default();
        let article = ConventionArticle::new("a1", "Title", "Content");
        s.update(&[article], ConventionType::International);
        assert_eq!(s.total_articles, 1);
        assert_eq!(s.binding, 1);
    }
}
