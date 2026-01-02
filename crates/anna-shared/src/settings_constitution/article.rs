// v0.0.725: Constitution Article and Clause (Phase 301)

use serde::{Deserialize, Serialize};

/// Constitution article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub number: u32,
    /// Ratified
    pub ratified: bool,
}

impl ConstitutionArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            number: 0,
            ratified: false,
        }
    }

    /// Set number
    pub fn number(mut self, n: u32) -> Self {
        self.number = n;
        self
    }

    /// Ratify article
    pub fn ratify(&mut self) {
        self.ratified = true;
    }

    /// Repeal article
    pub fn repeal(&mut self) {
        self.ratified = false;
    }
}

/// Constitution clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionClause {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Article ID
    pub article_id: String,
}

impl ConstitutionClause {
    /// Create new clause
    pub fn new(key: impl Into<String>, value: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            article_id: article_id.into(),
        }
    }
}
