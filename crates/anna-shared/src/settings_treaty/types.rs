// v0.0.727: Settings Treaty (Phase 303)
// International agreement for settings governance - Types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Treaty type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TreatyType {
    /// Bilateral treaty
    #[default]
    Bilateral,
    /// Multilateral treaty
    Multilateral,
    /// Framework treaty
    Framework,
    /// Protocol treaty
    Protocol,
}

impl std::fmt::Display for TreatyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bilateral => write!(f, "bilateral"),
            Self::Multilateral => write!(f, "multilateral"),
            Self::Framework => write!(f, "framework"),
            Self::Protocol => write!(f, "protocol"),
        }
    }
}

/// Treaty status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TreatyStatus {
    /// Negotiating status
    #[default]
    Negotiating,
    /// Signed status
    Signed,
    /// Ratified status
    Ratified,
    /// Terminated status
    Terminated,
}

impl std::fmt::Display for TreatyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negotiating => write!(f, "negotiating"),
            Self::Signed => write!(f, "signed"),
            Self::Ratified => write!(f, "ratified"),
            Self::Terminated => write!(f, "terminated"),
        }
    }
}

/// Treaty config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatyConfig {
    /// Name
    pub name: String,
    /// Treaty type
    pub treaty_type: TreatyType,
    /// Status
    pub status: TreatyStatus,
    /// Max provisions
    pub max_provisions: usize,
}

impl TreatyConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            treaty_type: TreatyType::Bilateral,
            status: TreatyStatus::Negotiating,
            max_provisions: 100,
        }
    }

    /// Set type
    pub fn treaty_type(mut self, tt: TreatyType) -> Self {
        self.treaty_type = tt;
        self
    }

    /// Set status
    pub fn status(mut self, s: TreatyStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max provisions
    pub fn max_provisions(mut self, max: usize) -> Self {
        self.max_provisions = max;
        self
    }
}

impl Default for TreatyConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Treaty provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatyProvision {
    /// Provision ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Article number
    pub article: u32,
    /// In force
    pub in_force: bool,
}

impl TreatyProvision {
    /// Create new provision
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            article: 0,
            in_force: false,
        }
    }

    /// Set article
    pub fn article(mut self, a: u32) -> Self {
        self.article = a;
        self
    }

    /// Enter into force
    pub fn enter_force(&mut self) {
        self.in_force = true;
    }

    /// Terminate
    pub fn terminate(&mut self) {
        self.in_force = false;
    }
}

/// Treaty signatory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatySignatory {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Provision ID
    pub provision_id: String,
}

impl TreatySignatory {
    /// Create new signatory
    pub fn new(key: impl Into<String>, name: impl Into<String>, provision_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            provision_id: provision_id.into(),
        }
    }
}

/// Treaty stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TreatyStats {
    /// Total provisions
    pub total_provisions: usize,
    /// In force provisions
    pub in_force: usize,
    /// Ratified count
    pub ratified_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl TreatyStats {
    /// Update from provisions
    pub fn update(&mut self, provisions: &[TreatyProvision], treaty_type: TreatyType) {
        self.total_provisions = provisions.len();
        self.in_force = provisions.iter().filter(|p| p.in_force).count();
        *self.by_type.entry(treaty_type.to_string()).or_insert(0) += 1;
    }

    /// In force rate
    pub fn in_force_rate(&self) -> f64 {
        if self.total_provisions == 0 { 0.0 } else { self.in_force as f64 / self.total_provisions as f64 * 100.0 }
    }
}
