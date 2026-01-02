// v0.0.738: Settings Confederation Stats
// Statistics for confederation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ConfederationType;
use super::article::ConfederationArticle;

/// Confederation stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfederationStats {
    /// Total articles
    pub total_articles: usize,
    /// Voluntary articles
    pub voluntary: usize,
    /// Functional count
    pub functional_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ConfederationStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[ConfederationArticle], confederation_type: ConfederationType) {
        self.total_articles = articles.len();
        self.voluntary = articles.iter().filter(|a| a.voluntary).count();
        *self.by_type.entry(confederation_type.to_string()).or_insert(0) += 1;
    }

    /// Voluntary rate
    pub fn voluntary_rate(&self) -> f64 {
        if self.total_articles == 0 { 0.0 } else { self.voluntary as f64 / self.total_articles as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = ConfederationStats::default();
        let article = ConfederationArticle::new("a1", "Title", "Content");
        s.update(&[article], ConfederationType::Sovereign);
        assert_eq!(s.total_articles, 1);
        assert_eq!(s.voluntary, 1);
    }
}
