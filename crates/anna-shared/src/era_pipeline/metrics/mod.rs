//! Honest Metrics (Part F) - v0.0.441.
//!
//! Kill fake success metrics.
//!
//! Ticket is RESOLVED only if:
//! - can_answer=true
//! - missing=[]
//! - answer delivered
//!
//! Everything else is NOT resolved.
//! Stats must reflect this or the system is lying.

mod tracker;
mod types;
mod validator;

// Re-export all public types
pub use tracker::{HonestMetrics, MetricsSummary};
pub use types::{ResolutionCriteria, ResolutionReason, ResolutionStatus};
pub use validator::{validate_resolution, DEFAULT_CONFIDENCE_THRESHOLD};
