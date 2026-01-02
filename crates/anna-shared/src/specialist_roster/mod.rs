//! Specialist Roster - Phase 87
//!
//! Manages specialist identities with persistent human names.
//! VISION.md: "Permanent names for each person (human names, diverse)"

mod formatting;
mod management;
mod names;
mod types;

#[cfg(test)]
mod tests;

// Re-export types
pub use types::{Department, SpecialistLevel, SpecialistProfile};

// Re-export management
pub use management::SpecialistRoster;

// Re-export names
pub use names::{get_specialist_name, SPECIALIST_NAMES};

// Re-export formatting functions
pub use formatting::{
    format_roster_compact, format_roster_oneline, format_specialist_roster,
    is_specialist_roster_query, roster_fun_fact,
};
