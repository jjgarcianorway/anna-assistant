//! Proactive health tips from system state (v0.0.244).
//!
//! Generates idle tips based on actual system health - disk usage,
//! memory pressure, failed services, etc. These tips surface during
//! REPL idle time to help users before they even ask.
//!
//! v0.0.244: Initial implementation.
//! v0.0.285: Added telemetry-based trend tips.

mod delta_tips;
mod snapshot_checks;
mod telemetry_tips;

// Re-export public API
pub use snapshot_checks::generate_health_tips;
pub use telemetry_tips::generate_telemetry_tips;
