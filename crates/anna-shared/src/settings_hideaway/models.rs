// v0.0.786: Settings Hideaway (Phase 362)
// Core data models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::{HideawayType, HideawayStatus};

/// Hideaway config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HideawayConfig {
    /// Name
    pub name: String,
    /// Hideaway type
    pub hideaway_type: HideawayType,
    /// Status
    pub status: HideawayStatus,
    /// Max occupants
    pub max_occupants: usize,
}

impl HideawayConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hideaway_type: HideawayType::Secret,
            status: HideawayStatus::Secluded,
            max_occupants: 100,
        }
    }

    /// Set type
    pub fn hideaway_type(mut self, ht: HideawayType) -> Self {
        self.hideaway_type = ht;
        self
    }

    /// Set status
    pub fn status(mut self, s: HideawayStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max occupants
    pub fn max_occupants(mut self, max: usize) -> Self {
        self.max_occupants = max;
        self
    }
}

impl Default for HideawayConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Hideaway occupant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HideawayOccupant {
    /// Occupant ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Nook number
    pub nook: u32,
    /// Hidden
    pub hidden: bool,
}

impl HideawayOccupant {
    /// Create new occupant
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            nook: 0,
            hidden: true,
        }
    }

    /// Set nook
    pub fn nook(mut self, n: u32) -> Self {
        self.nook = n;
        self
    }

    /// Make hidden
    pub fn make_hidden(&mut self) {
        self.hidden = true;
    }

    /// Make visible
    pub fn make_visible(&mut self) {
        self.hidden = false;
    }
}

/// Hideaway guardian
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HideawayGuardian {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Occupant ID
    pub occupant_id: String,
}

impl HideawayGuardian {
    /// Create new guardian
    pub fn new(key: impl Into<String>, name: impl Into<String>, occupant_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            occupant_id: occupant_id.into(),
        }
    }
}

/// Hideaway stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HideawayStats {
    /// Total occupants
    pub total_occupants: usize,
    /// Hidden occupants
    pub hidden: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl HideawayStats {
    /// Update from occupants
    pub fn update(&mut self, occupants: &[HideawayOccupant], hideaway_type: HideawayType) {
        self.total_occupants = occupants.len();
        self.hidden = occupants.iter().filter(|o| o.hidden).count();
        *self.by_type.entry(hideaway_type.to_string()).or_insert(0) += 1;
    }

    /// Hidden rate
    pub fn hidden_rate(&self) -> f64 {
        if self.total_occupants == 0 { 0.0 } else { self.hidden as f64 / self.total_occupants as f64 * 100.0 }
    }
}
