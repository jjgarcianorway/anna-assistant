// v0.0.725: Settings Constitution Module (Phase 301)
// Supreme law for settings governance

mod types;
mod config;
mod article;
mod stats;
mod constitution;
mod registry;
mod helpers;
mod tests;

// Re-export all public types
pub use types::{ConstitutionType, ConstitutionBranch};
pub use config::ConstitutionConfig;
pub use article::{ConstitutionArticle, ConstitutionClause};
pub use stats::ConstitutionStats;
pub use constitution::SettingsConstitution;
pub use registry::ConstitutionRegistry;
pub use helpers::{format_constitution_registry, is_constitution_query, constitution_fun_fact};
