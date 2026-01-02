//! Staff relationships and dynamics (v0.0.262).
//!
//! Defines mentor-mentee, friendship, and cross-team collaboration
//! relationships between staff members for richer dialogue.

mod data;
mod phrases;
mod queries;
mod types;

// Re-export all public items
pub use phrases::{escalation_phrase, mention_phrase, senior_response_phrase};
pub use queries::{
    get_collaborators, get_friends, get_mentor, get_rival, get_shift_buddies, have_relationship,
    relationships_for,
};
pub use types::{Relationship, RelationType};
