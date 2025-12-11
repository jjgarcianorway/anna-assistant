//! Recipe telemetry for Anna's learning system.
//! v0.0.418: Tracks recipe usage, resolution sources, and learning progress.
//!
//! Stats tracked:
//! - recipes_total: Total recipes in storage
//! - recipes_active: Active recipes
//! - tickets_resolved_by_recipes: Tickets resolved without LLM
//! - tickets_resolved_by_specialists: Tickets requiring LLM
//! - learning_events: Recipe creation/update events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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

/// Recipe telemetry tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeTelemetry {
    /// Resolution events (rolling window)
    pub resolutions: Vec<ResolutionEvent>,
    /// Learning events
    pub learning_events: Vec<LearningEvent>,
    /// Aggregated stats
    pub stats: TelemetryStats,
    /// Maximum events to keep
    #[serde(skip)]
    max_events: usize,
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

impl Default for RecipeTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeTelemetry {
    pub fn new() -> Self {
        Self {
            resolutions: Vec::new(),
            learning_events: Vec::new(),
            stats: TelemetryStats::default(),
            max_events: 1000,
        }
    }

    /// Load telemetry from file.
    pub fn load(path: &PathBuf) -> Self {
        if path.exists() {
            std::fs::read_to_string(path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::new()
        }
    }

    /// Save telemetry to file.
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Record a resolution event.
    pub fn record_resolution(&mut self, event: ResolutionEvent) {
        // Update aggregated stats
        self.stats.total_resolutions += 1;
        match event.source {
            ResolutionSource::Recipe => self.stats.by_recipe += 1,
            ResolutionSource::Specialist => self.stats.by_specialist += 1,
            ResolutionSource::IntentHandler => self.stats.by_intent_handler += 1,
            ResolutionSource::FastPath => self.stats.by_fast_path += 1,
            ResolutionSource::Failed => self.stats.failed += 1,
        }

        // Update domain stats
        if let Some(domain) = &event.domain {
            let domain_stats = self.stats.by_domain.entry(domain.clone()).or_default();
            domain_stats.total += 1;
            match event.source {
                ResolutionSource::Recipe | ResolutionSource::FastPath => {
                    domain_stats.by_recipe += 1;
                }
                ResolutionSource::Specialist | ResolutionSource::IntentHandler => {
                    domain_stats.by_specialist += 1;
                }
                _ => {}
            }
        }

        // Store event
        self.resolutions.push(event);

        // Trim if needed
        if self.resolutions.len() > self.max_events {
            self.resolutions.drain(0..self.resolutions.len() - self.max_events);
        }
    }

    /// Record a learning event.
    pub fn record_learning(&mut self, event: LearningEvent) {
        match event.event_type {
            LearningEventType::RecipeCreated => self.stats.recipes_created += 1,
            LearningEventType::RecipeDisabled => self.stats.recipes_disabled += 1,
            _ => {}
        }

        self.learning_events.push(event);

        // Trim learning events
        if self.learning_events.len() > self.max_events {
            self.learning_events
                .drain(0..self.learning_events.len() - self.max_events);
        }
    }

    /// Get recipe self-reliance percentage.
    pub fn self_reliance_rate(&self) -> f32 {
        if self.stats.total_resolutions == 0 {
            return 0.0;
        }
        let self_reliant = self.stats.by_recipe + self.stats.by_fast_path + self.stats.by_intent_handler;
        self_reliant as f32 / self.stats.total_resolutions as f32 * 100.0
    }

    /// Get success rate (non-failed resolutions).
    pub fn success_rate(&self) -> f32 {
        if self.stats.total_resolutions == 0 {
            return 0.0;
        }
        let successful = self.stats.total_resolutions - self.stats.failed;
        successful as f32 / self.stats.total_resolutions as f32 * 100.0
    }

    /// Generate summary string for display.
    pub fn summary(&self, active_recipes: usize, disabled_recipes: usize) -> String {
        format!(
            "Learning: {} recipes ({} active, {} disabled), {} tickets resolved by recipes ({:.0}% self-reliance)",
            active_recipes + disabled_recipes,
            active_recipes,
            disabled_recipes,
            self.stats.by_recipe + self.stats.by_fast_path,
            self.self_reliance_rate()
        )
    }

    /// Get recent resolutions (last N).
    pub fn recent_resolutions(&self, n: usize) -> &[ResolutionEvent] {
        let start = self.resolutions.len().saturating_sub(n);
        &self.resolutions[start..]
    }

    /// Get recent learning events (last N).
    pub fn recent_learning(&self, n: usize) -> &[LearningEvent] {
        let start = self.learning_events.len().saturating_sub(n);
        &self.learning_events[start..]
    }

    /// Get stats for a specific domain.
    pub fn domain_stats(&self, domain: &str) -> Option<&DomainStats> {
        self.stats.by_domain.get(domain)
    }
}

/// Helper to record a resolution.
pub fn record_resolution(
    telemetry: &mut RecipeTelemetry,
    ticket_id: &str,
    source: ResolutionSource,
    recipe_id: Option<&str>,
    intent: Option<&str>,
    domain: Option<&str>,
    duration_ms: u64,
) {
    telemetry.record_resolution(ResolutionEvent {
        timestamp: Utc::now(),
        ticket_id: ticket_id.to_string(),
        source,
        recipe_id: recipe_id.map(String::from),
        intent: intent.map(String::from),
        domain: domain.map(String::from),
        duration_ms,
    });
}

/// Helper to record a learning event.
pub fn record_learning(
    telemetry: &mut RecipeTelemetry,
    event_type: LearningEventType,
    recipe_id: &str,
    from_ticket_id: Option<&str>,
    details: &str,
) {
    telemetry.record_learning(LearningEvent {
        timestamp: Utc::now(),
        event_type,
        recipe_id: recipe_id.to_string(),
        from_ticket_id: from_ticket_id.map(String::from),
        details: details.to_string(),
    });
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

impl RecipeTelemetry {
    pub fn detailed_stats(&self, active_recipes: usize, disabled_recipes: usize) -> DetailedStats {
        DetailedStats {
            total_recipes: active_recipes + disabled_recipes,
            active_recipes,
            disabled_recipes,
            total_resolutions: self.stats.total_resolutions,
            by_recipe: self.stats.by_recipe,
            by_specialist: self.stats.by_specialist,
            by_intent_handler: self.stats.by_intent_handler,
            by_fast_path: self.stats.by_fast_path,
            failed: self.stats.failed,
            self_reliance_rate: self.self_reliance_rate(),
            success_rate: self.success_rate(),
            recipes_created: self.stats.recipes_created,
            domains: self.stats.by_domain.keys().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_basic() {
        let mut telemetry = RecipeTelemetry::new();

        record_resolution(
            &mut telemetry,
            "ticket1",
            ResolutionSource::Recipe,
            Some("vim_syntax"),
            Some("configure_editor"),
            Some("desktop"),
            100,
        );

        record_resolution(
            &mut telemetry,
            "ticket2",
            ResolutionSource::Specialist,
            None,
            Some("check_disk"),
            Some("storage"),
            500,
        );

        assert_eq!(telemetry.stats.total_resolutions, 2);
        assert_eq!(telemetry.stats.by_recipe, 1);
        assert_eq!(telemetry.stats.by_specialist, 1);
        assert_eq!(telemetry.self_reliance_rate(), 50.0);
    }

    #[test]
    fn test_learning_events() {
        let mut telemetry = RecipeTelemetry::new();

        record_learning(
            &mut telemetry,
            LearningEventType::RecipeCreated,
            "vim_syntax",
            Some("ticket1"),
            "Created from successful ticket",
        );

        assert_eq!(telemetry.stats.recipes_created, 1);
        assert_eq!(telemetry.learning_events.len(), 1);
    }

    #[test]
    fn test_summary() {
        let mut telemetry = RecipeTelemetry::new();

        for i in 0..10 {
            let source = if i < 6 {
                ResolutionSource::Recipe
            } else {
                ResolutionSource::Specialist
            };
            record_resolution(&mut telemetry, &format!("t{}", i), source, None, None, None, 100);
        }

        let summary = telemetry.summary(5, 1);
        assert!(summary.contains("6 recipes"));
        assert!(summary.contains("5 active"));
        assert!(summary.contains("1 disabled"));
        assert!(summary.contains("60%")); // 6/10 = 60% self-reliance
    }
}
