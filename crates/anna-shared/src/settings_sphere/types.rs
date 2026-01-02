// v0.0.741: Settings Sphere Types (Phase 317)
// Basic types and enums for settings sphere

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sphere type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SphereType {
    /// Influence sphere
    #[default]
    Influence,
    /// Co-prosperity sphere
    CoProsperity,
    /// Interest sphere
    Interest,
    /// Security sphere
    Security,
}

impl std::fmt::Display for SphereType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Influence => write!(f, "influence"),
            Self::CoProsperity => write!(f, "co-prosperity"),
            Self::Interest => write!(f, "interest"),
            Self::Security => write!(f, "security"),
        }
    }
}

/// Sphere status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SphereStatus {
    /// Expanding status
    #[default]
    Expanding,
    /// Stable status
    Stable,
    /// Contested status
    Contested,
    /// Declining status
    Declining,
}

impl std::fmt::Display for SphereStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expanding => write!(f, "expanding"),
            Self::Stable => write!(f, "stable"),
            Self::Contested => write!(f, "contested"),
            Self::Declining => write!(f, "declining"),
        }
    }
}

/// Sphere config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphereConfig {
    /// Name
    pub name: String,
    /// Sphere type
    pub sphere_type: SphereType,
    /// Status
    pub status: SphereStatus,
    /// Max interests
    pub max_interests: usize,
}

impl SphereConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sphere_type: SphereType::Influence,
            status: SphereStatus::Expanding,
            max_interests: 100,
        }
    }

    /// Set type
    pub fn sphere_type(mut self, st: SphereType) -> Self {
        self.sphere_type = st;
        self
    }

    /// Set status
    pub fn status(mut self, s: SphereStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max interests
    pub fn max_interests(mut self, max: usize) -> Self {
        self.max_interests = max;
        self
    }
}

impl Default for SphereConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Sphere interest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphereInterest {
    /// Interest ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Radius level
    pub radius: u32,
    /// Core interest
    pub core: bool,
}

impl SphereInterest {
    /// Create new interest
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            radius: 0,
            core: true,
        }
    }

    /// Set radius
    pub fn radius(mut self, r: u32) -> Self {
        self.radius = r;
        self
    }

    /// Make core
    pub fn make_core(&mut self) {
        self.core = true;
    }

    /// Make peripheral
    pub fn make_peripheral(&mut self) {
        self.core = false;
    }
}

/// Sphere entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphereEntity {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Interest ID
    pub interest_id: String,
}

impl SphereEntity {
    /// Create new entity
    pub fn new(key: impl Into<String>, name: impl Into<String>, interest_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            interest_id: interest_id.into(),
        }
    }
}

/// Sphere stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SphereStats {
    /// Total interests
    pub total_interests: usize,
    /// Core interests
    pub core: usize,
    /// Stable count
    pub stable_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl SphereStats {
    /// Update from interests
    pub fn update(&mut self, interests: &[SphereInterest], sphere_type: SphereType) {
        self.total_interests = interests.len();
        self.core = interests.iter().filter(|i| i.core).count();
        *self.by_type.entry(sphere_type.to_string()).or_insert(0) += 1;
    }

    /// Core rate
    pub fn core_rate(&self) -> f64 {
        if self.total_interests == 0 { 0.0 } else { self.core as f64 / self.total_interests as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_type_display() {
        assert_eq!(format!("{}", SphereType::Influence), "influence");
        assert_eq!(format!("{}", SphereType::Interest), "interest");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", SphereStatus::Expanding), "expanding");
        assert_eq!(format!("{}", SphereStatus::Stable), "stable");
    }

    #[test]
    fn test_config_new() {
        let c = SphereConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = SphereConfig::new("test")
            .sphere_type(SphereType::Security)
            .status(SphereStatus::Contested);
        assert_eq!(c.sphere_type, SphereType::Security);
        assert_eq!(c.status, SphereStatus::Contested);
    }

    #[test]
    fn test_interest_new() {
        let i = SphereInterest::new("i1", "Title", "Content");
        assert_eq!(i.id, "i1");
    }

    #[test]
    fn test_interest_builder() {
        let i = SphereInterest::new("i1", "Title", "Content")
            .radius(1);
        assert_eq!(i.radius, 1);
    }

    #[test]
    fn test_interest_core() {
        let mut i = SphereInterest::new("i1", "Title", "Content");
        i.make_peripheral();
        assert!(!i.core);
        i.make_core();
        assert!(i.core);
    }

    #[test]
    fn test_entity_new() {
        let e = SphereEntity::new("key", "name", "i1");
        assert_eq!(e.interest_id, "i1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = SphereStats::default();
        let interest = SphereInterest::new("i1", "Title", "Content");
        s.update(&[interest], SphereType::Influence);
        assert_eq!(s.total_interests, 1);
        assert_eq!(s.core, 1);
    }
}
