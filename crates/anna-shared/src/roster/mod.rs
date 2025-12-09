//! Humanized IT Department Roster with stable person profiles (v0.0.243).
//!
//! Each specialist has a deterministic identity: name, role, team, tier.
//! No randomness - same (Team, Tier) always maps to the same person.
//!
//! v0.0.42: Updated pinned names per user specification.
//! v0.0.182: Modularized into domain-focused submodules.
//! v0.0.243: Added personality traits and dialogue for each staff member.

mod data;
pub mod personality;
mod profile;
mod shift;
mod tests;
mod tier;

// Re-export main types and functions
pub use data::{all_persons, person_by_display_name, person_by_id, person_for, team_roster};
pub use personality::{get_greeting, get_success, get_uncertain, personality_for, Personality};
pub use profile::PersonProfile;
pub use shift::Shift;
pub use tier::Tier;
