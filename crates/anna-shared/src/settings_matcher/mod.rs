// v0.0.687: Settings Matcher Module (Phase 263)
// Match settings against patterns and rules

mod items;
mod matcher;
mod registry;
mod stats;
mod types;
mod utils;

// Re-export public types and functions
pub use items::{MatchItem, MatchResult};
pub use matcher::SettingsMatcher;
pub use registry::{format_matcher_registry, MatcherRegistry};
pub use stats::MatcherStats;
pub use types::{MatchRule, MatchTarget, MatchType, MatcherConfig};
pub use utils::{is_matcher_query, matcher_fun_fact};
