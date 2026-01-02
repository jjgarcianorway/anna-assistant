// v0.0.527: Skill Proficiency Tracker (Phase 103)
// Module declarations and re-exports

mod formatting;
mod tracker;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to maintain API compatibility
pub use formatting::{format_skill, format_skill_compact, format_skill_oneline, format_tracker_summary};
pub use tracker::SkillProficiencyTracker;
pub use types::{ProficiencyLevel, SkillDomain, SkillRecord};
pub use utils::{is_skill_query, skill_fun_fact};
