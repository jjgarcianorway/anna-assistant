// v0.0.745: Settings Territory - Ordinance
// Territory ordinance management

use serde::{Deserialize, Serialize};

/// Territory ordinance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryOrdinance {
    /// Ordinance ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// District number
    pub district: u32,
    /// Enforced
    pub enforced: bool,
}

impl TerritoryOrdinance {
    /// Create new ordinance
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            district: 0,
            enforced: true,
        }
    }

    /// Set district
    pub fn district(mut self, d: u32) -> Self {
        self.district = d;
        self
    }

    /// Make enforced
    pub fn make_enforced(&mut self) {
        self.enforced = true;
    }

    /// Make suspended
    pub fn make_suspended(&mut self) {
        self.enforced = false;
    }
}
