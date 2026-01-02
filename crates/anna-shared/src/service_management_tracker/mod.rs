//! Service Management Tracker - Phase 81
//!
//! Tracks service operations (start, stop, restart, enable, disable) by Anna.
//! VISION.md mentions Anna being able to restart services and manage daemons.

pub mod formatting;
pub mod queries;
pub mod tracker;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use formatting::{
    format_service_tracker, format_service_tracker_compact, format_service_tracker_oneline,
};
pub use queries::{is_service_tracker_query, service_fun_fact};
pub use tracker::ServiceTracker;
pub use types::{OperationResult, ServiceOperation, ServiceRecord};
