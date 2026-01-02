//! Reliability Metrics (v0.0.444).
//!
//! Fix the lying stats problem. Stats must reflect reality.
//!
//! Components:
//! - canonical_outcome: Single source of truth for ticket outcomes
//! - request_metrics: Per-request tracking with evidence coverage
//! - aggregate_stats: Real reliability metrics (not fake staff performance)
//! - model_inventory: Accurate model/probe inventory (no duplicates)
//!
//! Rules:
//! - Only ANSWERED_VERIFIED counts as "resolved"
//! - Stats never show 100% if failures exist
//! - Model inventory is deduplicated and shows ownership

pub mod aggregate_stats;
pub mod aggregate_stats_calc;
pub mod aggregate_stats_display;
pub mod aggregate_stats_types;
pub mod canonical_outcome;
pub mod model_inventory;
pub mod model_inventory_probes;
pub mod model_inventory_types;
pub mod request_metrics;
pub mod request_metrics_builder;
pub mod request_metrics_store;
pub mod request_metrics_types;

#[cfg(test)]
pub mod tests;

#[cfg(test)]
pub mod tests_outcomes;

#[cfg(test)]
pub mod tests_inventory;

// Re-exports for convenience
pub use aggregate_stats::{compute_stats, ReliabilityStats, TopicStats};
pub use canonical_outcome::{
    from_ticket_integrity_outcome, from_ticket_state_outcome, CanonicalOutcome, OutcomeConditions,
};
pub use model_inventory::{
    default_probe_inventory, ConfiguredModels, ModelEntry, ModelInventory, ModelOwner, ProbeEntry,
    ProbeInventory,
};
pub use request_metrics::{ModelsUsed, RequestMetrics, RequestMetricsBuilder, RequestMetricsStore};

/// Version of the reliability metrics system.
pub const RELIABILITY_VERSION: &str = "0.0.444";
