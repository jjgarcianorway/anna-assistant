//! Recovery framework for system repair and rescue
//!
//! Phase 0.6: Recovery Framework - Main module
//! Citation: [archwiki:System_maintenance#Backup]

pub mod chroot;
pub mod parser;
pub mod types;

pub use chroot::{detect_chroot, is_chroot_candidate, prepare_chroot, cleanup_chroot};
pub use parser::{load_all_plans, load_plan, load_plan_from_file};
pub use types::{RecoveryPlan, RecoveryStep, RecoveryResult, StepResult, RollbackMetadata};
