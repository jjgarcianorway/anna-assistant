// v0.0.725: Constitution Stats (Phase 301)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::article::ConstitutionArticle;
use super::types::ConstitutionType;

/// Constitution stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstitutionStats {
    /// Total articles
    pub total_articles: usize,
    /// Ratified articles
    pub ratified: usize,
    /// Executive count
    pub executive_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ConstitutionStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[ConstitutionArticle], constitution_type: ConstitutionType) {
        self.total_articles = articles.len();
        self.ratified = articles.iter().filter(|a| a.ratified).count();
        *self.by_type.entry(constitution_type.to_string()).or_insert(0) += 1;
    }

    /// Ratified rate
    pub fn ratified_rate(&self) -> f64 {
        if self.total_articles == 0 { 0.0 } else { self.ratified as f64 / self.total_articles as f64 * 100.0 }
    }
}
