// v0.0.723: Settings Statute Types (Phase 299)
// Type definitions for statute system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statute type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StatuteType {
    /// General statute
    #[default]
    General,
    /// Criminal statute
    Criminal,
    /// Civil statute
    Civil,
    /// Administrative statute
    Administrative,
}

impl std::fmt::Display for StatuteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General => write!(f, "general"),
            Self::Criminal => write!(f, "criminal"),
            Self::Civil => write!(f, "civil"),
            Self::Administrative => write!(f, "administrative"),
        }
    }
}

/// Statute scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StatuteScope {
    /// Federal scope
    #[default]
    Federal,
    /// State scope
    State,
    /// Local scope
    Local,
    /// International scope
    International,
}

impl std::fmt::Display for StatuteScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Federal => write!(f, "federal"),
            Self::State => write!(f, "state"),
            Self::Local => write!(f, "local"),
            Self::International => write!(f, "international"),
        }
    }
}

/// Statute config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatuteConfig {
    /// Name
    pub name: String,
    /// Statute type
    pub statute_type: StatuteType,
    /// Scope
    pub scope: StatuteScope,
    /// Max statutes
    pub max_statutes: usize,
}

impl StatuteConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            statute_type: StatuteType::General,
            scope: StatuteScope::Federal,
            max_statutes: 200,
        }
    }

    /// Set type
    pub fn statute_type(mut self, st: StatuteType) -> Self {
        self.statute_type = st;
        self
    }

    /// Set scope
    pub fn scope(mut self, s: StatuteScope) -> Self {
        self.scope = s;
        self
    }

    /// Set max statutes
    pub fn max_statutes(mut self, max: usize) -> Self {
        self.max_statutes = max;
        self
    }
}

impl Default for StatuteConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Statute article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatuteArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub number: String,
    /// Enacted
    pub enacted: bool,
}

impl StatuteArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            number: String::new(),
            enacted: false,
        }
    }

    /// Set number
    pub fn number(mut self, n: impl Into<String>) -> Self {
        self.number = n.into();
        self
    }

    /// Enact article
    pub fn enact(&mut self) {
        self.enacted = true;
    }

    /// Repeal article
    pub fn repeal(&mut self) {
        self.enacted = false;
    }
}

/// Statute subsection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatuteSubsection {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Article ID
    pub article_id: String,
}

impl StatuteSubsection {
    /// Create new subsection
    pub fn new(key: impl Into<String>, value: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            article_id: article_id.into(),
        }
    }
}

/// Statute stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatuteStats {
    /// Total statutes
    pub total_statutes: usize,
    /// Enacted statutes
    pub enacted: usize,
    /// Federal count
    pub federal_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl StatuteStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[StatuteArticle], statute_type: StatuteType) {
        self.total_statutes = articles.len();
        self.enacted = articles.iter().filter(|a| a.enacted).count();
        *self.by_type.entry(statute_type.to_string()).or_insert(0) += 1;
    }

    /// Enacted rate
    pub fn enacted_rate(&self) -> f64 {
        if self.total_statutes == 0 { 0.0 } else { self.enacted as f64 / self.total_statutes as f64 * 100.0 }
    }
}
