//! Recipe implementation methods.
//!
//! This module contains the implementation of Recipe methods for creation,
//! usage tracking, and utility functions.

use chrono::Utc;

use super::{
    ConfirmationPolicy, PlanStep, Recipe, RecipeMatcher, RecipeMetrics, RecipePattern,
    RecipeStatus, SuccessCriteria,
};

impl Recipe {
    /// Create a new recipe with default metrics
    pub fn new(
        id: String,
        domain: String,
        intent: String,
        pattern: RecipePattern,
        matcher: RecipeMatcher,
        plan: Vec<PlanStep>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            version: 1,
            domain,
            intent,
            pattern,
            matcher,
            preconditions: vec![],
            plan,
            confirmation_policy: ConfirmationPolicy::Require,
            success_criteria: SuccessCriteria::default(),
            citations: vec![],
            metrics: RecipeMetrics::default(),
            status: RecipeStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if recipe is usable
    pub fn is_usable(&self) -> bool {
        self.status == RecipeStatus::Active
    }

    /// Record a successful use
    pub fn record_success(&mut self) {
        self.metrics.times_used += 1;
        self.metrics.last_used_at = Some(Utc::now());
        self.update_success_rate();
    }

    /// Record a failed use
    pub fn record_failure(&mut self) {
        self.metrics.times_used += 1;
        self.metrics.times_failed += 1;
        self.metrics.last_used_at = Some(Utc::now());
        self.update_success_rate();
        // Auto-disable if failure rate too high
        if self.should_auto_disable() {
            self.status = RecipeStatus::Disabled;
        }
    }

    fn update_success_rate(&mut self) {
        if self.metrics.times_used > 0 {
            let successes = self.metrics.times_used - self.metrics.times_failed;
            self.metrics.recent_success_rate =
                Some(successes as f32 / self.metrics.times_used as f32);
        }
    }

    fn should_auto_disable(&self) -> bool {
        // Disable if 3+ failures in last 10 uses
        self.metrics.times_used >= 10
            && self.metrics.times_failed >= 3
            && self.metrics.recent_success_rate.unwrap_or(1.0) < 0.7
    }

    /// Check if recipe has mutating steps
    pub fn has_mutating_steps(&self) -> bool {
        self.plan.iter().any(|s| s.is_mutating())
    }

    /// Get files this recipe touches
    pub fn touched_files(&self) -> Vec<String> {
        self.plan
            .iter()
            .filter_map(|step| match step {
                PlanStep::BackupFile { path }
                | PlanStep::AppendLine { path, .. }
                | PlanStep::PrependLine { path, .. }
                | PlanStep::ReplaceLine { path, .. }
                | PlanStep::EnsureLine { path, .. }
                | PlanStep::RemoveLines { path, .. }
                | PlanStep::WriteFile { path, .. } => Some(path.clone()),
                _ => None,
            })
            .collect()
    }
}
