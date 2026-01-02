// v0.0.737: Settings Federation (Phase 313)
// Federal union for settings governance - Types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Federation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FederationType {
    /// Symmetric federation
    #[default]
    Symmetric,
    /// Asymmetric federation
    Asymmetric,
    /// Cooperative federation
    Cooperative,
    /// Dual federation
    Dual,
}

impl std::fmt::Display for FederationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Symmetric => write!(f, "symmetric"),
            Self::Asymmetric => write!(f, "asymmetric"),
            Self::Cooperative => write!(f, "cooperative"),
            Self::Dual => write!(f, "dual"),
        }
    }
}

/// Federation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FederationStatus {
    /// Constituting status
    #[default]
    Constituting,
    /// Established status
    Established,
    /// Reforming status
    Reforming,
    /// Dissolving status
    Dissolving,
}

impl std::fmt::Display for FederationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constituting => write!(f, "constituting"),
            Self::Established => write!(f, "established"),
            Self::Reforming => write!(f, "reforming"),
            Self::Dissolving => write!(f, "dissolving"),
        }
    }
}

/// Federation config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Name
    pub name: String,
    /// Federation type
    pub federation_type: FederationType,
    /// Status
    pub status: FederationStatus,
    /// Max articles
    pub max_articles: usize,
}

impl FederationConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            federation_type: FederationType::Symmetric,
            status: FederationStatus::Constituting,
            max_articles: 100,
        }
    }

    /// Set type
    pub fn federation_type(mut self, ft: FederationType) -> Self {
        self.federation_type = ft;
        self
    }

    /// Set status
    pub fn status(mut self, s: FederationStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max articles
    pub fn max_articles(mut self, max: usize) -> Self {
        self.max_articles = max;
        self
    }
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Federation article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationArticle {
    /// Article ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Constitutional
    pub constitutional: bool,
}

impl FederationArticle {
    /// Create new article
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            constitutional: false,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make constitutional
    pub fn make_constitutional(&mut self) {
        self.constitutional = true;
    }

    /// Make statutory
    pub fn make_statutory(&mut self) {
        self.constitutional = false;
    }
}

/// Federation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationState {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Article ID
    pub article_id: String,
}

impl FederationState {
    /// Create new state
    pub fn new(key: impl Into<String>, name: impl Into<String>, article_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            article_id: article_id.into(),
        }
    }
}

/// Federation stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FederationStats {
    /// Total articles
    pub total_articles: usize,
    /// Constitutional articles
    pub constitutional: usize,
    /// Established count
    pub established_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl FederationStats {
    /// Update from articles
    pub fn update(&mut self, articles: &[FederationArticle], federation_type: FederationType) {
        self.total_articles = articles.len();
        self.constitutional = articles.iter().filter(|a| a.constitutional).count();
        *self.by_type.entry(federation_type.to_string()).or_insert(0) += 1;
    }

    /// Constitutional rate
    pub fn constitutional_rate(&self) -> f64 {
        if self.total_articles == 0 { 0.0 } else { self.constitutional as f64 / self.total_articles as f64 * 100.0 }
    }
}
