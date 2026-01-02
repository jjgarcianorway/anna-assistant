//! Relationship types and structures (v0.0.262).

use serde::{Deserialize, Serialize};

/// Type of relationship between two staff members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    /// Senior mentors junior (within same team)
    Mentor,
    /// Cross-team friendship (similar interests)
    Friend,
    /// Friendly rivalry (competitive but respectful)
    Rival,
    /// Cross-team collaboration (complementary skills)
    Collaborator,
    /// Coffee buddies (same shift, hang out together)
    ShiftBuddy,
}

/// A relationship between two staff members
#[derive(Debug, Clone, Copy)]
pub struct Relationship {
    /// Source person ID
    pub from_id: &'static str,
    /// Target person ID
    pub to_id: &'static str,
    /// Type of relationship
    pub relation_type: RelationType,
}
