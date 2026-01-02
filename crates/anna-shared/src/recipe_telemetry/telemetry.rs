//! Recipe telemetry tracker implementation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::types::{
    DetailedStats, DomainStats, LearningEvent, LearningEventType, ResolutionEvent,
    ResolutionSource, TelemetryStats,
};

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
            self.resolutions
                .drain(0..self.resolutions.len() - self.max_events);
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
        let self_reliant =
            self.stats.by_recipe + self.stats.by_fast_path + self.stats.by_intent_handler;
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

    /// Get detailed stats for display.
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
