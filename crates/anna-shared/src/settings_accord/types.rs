// v0.0.730: Settings Accord (Phase 306)
// Type definitions for settings accord

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Accord type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AccordType {
    /// Peace accord
    #[default]
    Peace,
    /// Trade accord
    Trade,
    /// Framework accord
    Framework,
    /// Settlement accord
    Settlement,
}

impl std::fmt::Display for AccordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Peace => write!(f, "peace"),
            Self::Trade => write!(f, "trade"),
            Self::Framework => write!(f, "framework"),
            Self::Settlement => write!(f, "settlement"),
        }
    }
}

/// Accord status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AccordStatus {
    /// Preliminary status
    #[default]
    Preliminary,
    /// Final status
    Final,
    /// Implemented status
    Implemented,
    /// Voided status
    Voided,
}

impl std::fmt::Display for AccordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preliminary => write!(f, "preliminary"),
            Self::Final => write!(f, "final"),
            Self::Implemented => write!(f, "implemented"),
            Self::Voided => write!(f, "voided"),
        }
    }
}

/// Accord config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccordConfig {
    /// Name
    pub name: String,
    /// Accord type
    pub accord_type: AccordType,
    /// Status
    pub status: AccordStatus,
    /// Max provisions
    pub max_provisions: usize,
}

impl AccordConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            accord_type: AccordType::Peace,
            status: AccordStatus::Preliminary,
            max_provisions: 100,
        }
    }

    /// Set type
    pub fn accord_type(mut self, at: AccordType) -> Self {
        self.accord_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: AccordStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max provisions
    pub fn max_provisions(mut self, max: usize) -> Self {
        self.max_provisions = max;
        self
    }
}

impl Default for AccordConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Accord provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccordProvision {
    /// Provision ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Agreed
    pub agreed: bool,
}

impl AccordProvision {
    /// Create new provision
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            agreed: false,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Agree to provision
    pub fn agree(&mut self) {
        self.agreed = true;
    }

    /// Disagree to provision
    pub fn disagree(&mut self) {
        self.agreed = false;
    }
}

/// Accord signatory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccordSignatory {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Provision ID
    pub provision_id: String,
}

impl AccordSignatory {
    /// Create new signatory
    pub fn new(key: impl Into<String>, name: impl Into<String>, provision_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            provision_id: provision_id.into(),
        }
    }
}

/// Accord stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccordStats {
    /// Total provisions
    pub total_provisions: usize,
    /// Agreed provisions
    pub agreed: usize,
    /// Implemented count
    pub implemented_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AccordStats {
    /// Update from provisions
    pub fn update(&mut self, provisions: &[AccordProvision], accord_type: AccordType) {
        self.total_provisions = provisions.len();
        self.agreed = provisions.iter().filter(|p| p.agreed).count();
        *self.by_type.entry(accord_type.to_string()).or_insert(0) += 1;
    }

    /// Agreement rate
    pub fn agreement_rate(&self) -> f64 {
        if self.total_provisions == 0 { 0.0 } else { self.agreed as f64 / self.total_provisions as f64 * 100.0 }
    }
}
