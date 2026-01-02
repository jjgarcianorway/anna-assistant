// v0.0.570: Settings Constraints Module (Phase 146)
// Define and enforce rules/constraints on settings combinations

mod manager;
mod result;
mod types;
mod utils;

// Re-export public types and functions
pub use manager::ConstraintManager;
pub use result::{format_constraint_results, ConstraintCheckResult};
pub use types::{
    ConstraintSeverity, ConstraintType, ConstraintViolation, SettingsConstraint,
};
pub use utils::{constraint_fun_fact, is_constraint_query};
