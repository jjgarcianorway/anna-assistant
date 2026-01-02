// v0.0.710: Settings Brief (Phase 286)
// Executive briefs for settings overview

mod brief;
mod helpers;
mod registry;
mod stats;
#[cfg(test)]
mod tests;
mod types;

// Re-export all public types to maintain the same API
pub use brief::SettingsBrief;
pub use helpers::{brief_fun_fact, format_brief_registry, is_brief_query};
pub use registry::BriefRegistry;
pub use stats::BriefStats;
pub use types::{BriefAttachment, BriefConfig, BriefPoint, BriefScope, BriefType};
