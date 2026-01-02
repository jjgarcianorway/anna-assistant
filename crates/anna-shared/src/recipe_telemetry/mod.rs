//! Recipe telemetry for Anna's learning system.
//! v0.0.418: Tracks recipe usage, resolution sources, and learning progress.
//!
//! Stats tracked:
//! - recipes_total: Total recipes in storage
//! - recipes_active: Active recipes
//! - tickets_resolved_by_recipes: Tickets resolved without LLM
//! - tickets_resolved_by_specialists: Tickets requiring LLM
//! - learning_events: Recipe creation/update events

mod helpers;
mod telemetry;
#[cfg(test)]
mod tests;
mod types;

// Re-export all public items to maintain API compatibility
pub use helpers::{record_learning, record_resolution};
pub use telemetry::RecipeTelemetry;
pub use types::{
    DetailedStats, DomainStats, LearningEvent, LearningEventType, ResolutionEvent,
    ResolutionSource, TelemetryStats,
};
