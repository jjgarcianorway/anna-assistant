//! Workflow Automation Tracker - Phase 101
//!
//! Tracks automated workflows Anna creates and executes.
//! Enables complex multi-step automations.

mod types;
mod tracker;
mod formatting;

// Re-export public API
pub use types::{WorkflowTrigger, WorkflowStatus, WorkflowStep, WorkflowRecord};
pub use tracker::WorkflowAutomationTracker;
pub use formatting::{
    format_workflow_tracker,
    format_workflow_tracker_compact,
    format_workflow_tracker_oneline,
    is_workflow_query,
    workflow_fun_fact,
};
