//! Type definitions for recipe telemetry.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resolution source for a ticket.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionSource {
    /// Resolved by executing a learned recipe
    Recipe,
    /// Resolved by specialist LLM
    Specialist,
    /// Resolved by intent handler (deterministic)
    IntentHandler,
    /// Resolved by fast path (no LLM)
    FastPath,
    /// Failed to resolve
    Failed,
}

/// A single resolution event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionEvent {
    pub timestamp: DateTime<Utc>,
    pub ticket_id: String,
    pub source: ResolutionSource,
    pub recipe_id: Option<String>,
    pub intent: Option<String>,
    pub domain: Option<String>,
    pub duration_ms: u64,
}

/// A learning event (recipe created or updated).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: LearningEventType,
    pub recipe_id: String,
    pub from_ticket_id: Option<String>,
    pub details: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearningEventType {
    RecipeCreated,
    RecipeUpdated,
    RecipeDisabled,
    RecipeDeleted,
}

/// Aggregated telemetry stats.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetryStats {
    /// Total tickets resolved
    pub total_resolutions: u64,
    /// Tickets resolved by recipes
    pub by_recipe: u64,
    /// Tickets resolved by specialists
    pub by_specialist: u64,
    /// Tickets resolved by intent handlers
    pub by_intent_handler: u64,
    /// Tickets resolved by fast path
    pub by_fast_path: u64,
    /// Failed resolutions
    pub failed: u64,
    /// Total recipes created
    pub recipes_created: u64,
    /// Total recipes disabled
    pub recipes_disabled: u64,
    /// Stats per domain
    pub by_domain: HashMap<String, DomainStats>,
}

/// Stats for a single domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainStats {
    pub total: u64,
    pub by_recipe: u64,
    pub by_specialist: u64,
}

/// Detailed stats for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedStats {
    pub total_recipes: usize,
    pub active_recipes: usize,
    pub disabled_recipes: usize,
    pub total_resolutions: u64,
    pub by_recipe: u64,
    pub by_specialist: u64,
    pub by_intent_handler: u64,
    pub by_fast_path: u64,
    pub failed: u64,
    pub self_reliance_rate: f32,
    pub success_rate: f32,
    pub recipes_created: u64,
    pub domains: Vec<String>,
}
