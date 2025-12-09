//! Humanized IT Department Roster with stable person profiles (v0.0.182).
//!
//! Each specialist has a deterministic identity: name, role, team, tier.
//! No randomness - same (Team, Tier) always maps to the same person.
//!
//! v0.0.42: Updated pinned names per user specification.
//! v0.0.182: Modularized into domain-focused submodules.

mod data;
mod profile;
mod shift;
mod tests;
mod tier;

// Re-export main types and functions
pub use data::{all_persons, person_by_id, person_for, team_roster};
pub use profile::PersonProfile;
pub use shift::Shift;
pub use tier::Tier;
