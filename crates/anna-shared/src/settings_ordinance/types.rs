// v0.0.722: Settings Ordinance Types (Phase 298)
// Type definitions for settings ordinance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ordinance type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OrdinanceType {
    /// Municipal ordinance
    #[default]
    Municipal,
    /// Regional ordinance
    Regional,
    /// Local ordinance
    Local,
    /// Zoning ordinance
    Zoning,
}

impl std::fmt::Display for OrdinanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Municipal => write!(f, "municipal"),
            Self::Regional => write!(f, "regional"),
            Self::Local => write!(f, "local"),
            Self::Zoning => write!(f, "zoning"),
        }
    }
}

/// Ordinance jurisdiction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OrdinanceJurisdiction {
    /// City jurisdiction
    #[default]
    City,
    /// County jurisdiction
    County,
    /// District jurisdiction
    District,
    /// Zone jurisdiction
    Zone,
}

impl std::fmt::Display for OrdinanceJurisdiction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::City => write!(f, "city"),
            Self::County => write!(f, "county"),
            Self::District => write!(f, "district"),
            Self::Zone => write!(f, "zone"),
        }
    }
}

/// Ordinance config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdinanceConfig {
    /// Name
    pub name: String,
    /// Ordinance type
    pub ordinance_type: OrdinanceType,
    /// Jurisdiction
    pub jurisdiction: OrdinanceJurisdiction,
    /// Max ordinances
    pub max_ordinances: usize,
}

impl OrdinanceConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ordinance_type: OrdinanceType::Municipal,
            jurisdiction: OrdinanceJurisdiction::City,
            max_ordinances: 150,
        }
    }

    /// Set type
    pub fn ordinance_type(mut self, ot: OrdinanceType) -> Self {
        self.ordinance_type = ot;
        self
    }

    /// Set jurisdiction
    pub fn jurisdiction(mut self, j: OrdinanceJurisdiction) -> Self {
        self.jurisdiction = j;
        self
    }

    /// Set max ordinances
    pub fn max_ordinances(mut self, max: usize) -> Self {
        self.max_ordinances = max;
        self
    }
}

impl Default for OrdinanceConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Ordinance provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdinanceProvision {
    /// Provision ID
    pub id: String,
    /// Title
    pub title: String,
    /// Text
    pub text: String,
    /// Section number
    pub section: String,
    /// Effective
    pub effective: bool,
}

impl OrdinanceProvision {
    /// Create new provision
    pub fn new(id: impl Into<String>, title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            text: text.into(),
            section: String::new(),
            effective: false,
        }
    }

    /// Set section
    pub fn section(mut self, s: impl Into<String>) -> Self {
        self.section = s.into();
        self
    }

    /// Make effective
    pub fn make_effective(&mut self) {
        self.effective = true;
    }

    /// Make ineffective
    pub fn make_ineffective(&mut self) {
        self.effective = false;
    }
}

/// Ordinance amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdinanceAmendment {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Provision ID
    pub provision_id: String,
}

impl OrdinanceAmendment {
    /// Create new amendment
    pub fn new(key: impl Into<String>, value: impl Into<String>, provision_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            provision_id: provision_id.into(),
        }
    }
}

/// Ordinance stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrdinanceStats {
    /// Total ordinances
    pub total_ordinances: usize,
    /// Effective ordinances
    pub effective: usize,
    /// Municipal count
    pub municipal_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl OrdinanceStats {
    /// Update from provisions
    pub fn update(&mut self, provisions: &[OrdinanceProvision], ordinance_type: OrdinanceType) {
        self.total_ordinances = provisions.len();
        self.effective = provisions.iter().filter(|p| p.effective).count();
        if ordinance_type == OrdinanceType::Municipal {
            self.municipal_count = provisions.len();
        }
        *self.by_type.entry(ordinance_type.to_string()).or_insert(0) += 1;
    }

    /// Effective rate
    pub fn effective_rate(&self) -> f64 {
        if self.total_ordinances == 0 { 0.0 } else { self.effective as f64 / self.total_ordinances as f64 * 100.0 }
    }
}
