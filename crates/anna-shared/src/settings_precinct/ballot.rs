// v0.0.753: Settings Precinct Ballot (Phase 329)
// Ballot management for precincts

use serde::{Deserialize, Serialize};

/// Precinct ballot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecinctBallot {
    /// Ballot ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// District number
    pub district: u32,
    /// Certified
    pub certified: bool,
}

impl PrecinctBallot {
    /// Create new ballot
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            district: 0,
            certified: true,
        }
    }

    /// Set district
    pub fn district(mut self, d: u32) -> Self {
        self.district = d;
        self
    }

    /// Make certified
    pub fn make_certified(&mut self) {
        self.certified = true;
    }

    /// Make contested
    pub fn make_contested(&mut self) {
        self.certified = false;
    }
}
