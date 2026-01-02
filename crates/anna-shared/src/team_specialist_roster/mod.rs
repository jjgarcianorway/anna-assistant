// v0.0.528: Team Specialist Roster (Phase 104)
// Manages the full IT department roster with junior/senior specialists per VISION.md

mod formatting;
mod roster;
mod specialist;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use formatting::{
    format_roster_summary, format_specialist, format_specialist_compact,
    format_specialist_oneline, is_roster_query, roster_fun_fact,
};
pub use roster::TeamSpecialistRoster;
pub use specialist::Specialist;
pub use types::{AvailabilityStatus, Department, SeniorityLevel};
