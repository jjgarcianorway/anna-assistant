// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Compendium volume and article

use serde::{Deserialize, Serialize};

/// Compendium volume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompendiumVolume {
    /// Volume number
    pub number: usize,
    /// Title
    pub title: String,
    /// Subject
    pub subject: String,
    /// Articles
    pub articles: Vec<CompendiumArticle>,
}

impl CompendiumVolume {
    /// Create new volume
    pub fn new(number: usize, title: impl Into<String>) -> Self {
        Self {
            number,
            title: title.into(),
            subject: String::new(),
            articles: Vec::new(),
        }
    }

    /// Set subject
    pub fn subject(mut self, subj: impl Into<String>) -> Self {
        self.subject = subj.into();
        self
    }

    /// Add article
    pub fn add(&mut self, article: CompendiumArticle) {
        self.articles.push(article);
    }

    /// Article count
    pub fn article_count(&self) -> usize {
        self.articles.len()
    }
}

/// Compendium article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompendiumArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Keywords
    pub keywords: Vec<String>,
}

impl CompendiumArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            keywords: Vec::new(),
        }
    }

    /// Add keyword
    pub fn keyword(mut self, kw: impl Into<String>) -> Self {
        self.keywords.push(kw.into());
        self
    }
}
