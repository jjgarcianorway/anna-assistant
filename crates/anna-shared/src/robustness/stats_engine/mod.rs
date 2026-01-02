//! Truthful stats engine (v0.0.433).
//!
//! Stats that reflect actual outcomes, not any response.

mod engine;
mod types;

pub use engine::StatsEngine;
pub use types::{DepartmentStats, FailureRecord, StaffStats, TicketStats, TruthfulStats};
