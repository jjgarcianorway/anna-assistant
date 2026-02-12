//! Self-Improvement Loop - Anna tracks her own effectiveness and adjusts.
//!
//! Philosophy: Learn from outcomes, not assumptions. Track what works, adjust what doesn't.
//! NO HARDCODING: Evidence-based adjustments, not arbitrary thresholds.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

/// Tracks effectiveness of Anna's actions and suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessTracker {
    /// Suggestion acceptance rates by category
    pub suggestion_stats: HashMap<String, CategoryStats>,
    /// Auto-fix success rates by failure type
    pub autofix_stats: HashMap<String, CategoryStats>,
    /// Analysis module accuracy by module
    pub module_accuracy: HashMap<String, ModuleAccuracy>,
    /// Overall effectiveness score (0.0-1.0)
    pub overall_score: f32,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Statistics for a category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub total_attempts: u32,
    pub successful: u32,
    pub failed: u32,
    pub user_accepted: u32,
    pub user_rejected: u32,
    pub success_rate: f32,
    pub acceptance_rate: f32,
}

impl CategoryStats {
    fn new() -> Self {
        Self {
            total_attempts: 0,
            successful: 0,
            failed: 0,
            user_accepted: 0,
            user_rejected: 0,
            success_rate: 0.0,
            acceptance_rate: 0.0,
        }
    }

    fn update_rates(&mut self) {
        if self.total_attempts > 0 {
            self.success_rate = self.successful as f32 / self.total_attempts as f32;
        }
        let total_feedback = self.user_accepted + self.user_rejected;
        if total_feedback > 0 {
            self.acceptance_rate = self.user_accepted as f32 / total_feedback as f32;
        }
    }
}

/// Accuracy tracking for analysis modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAccuracy {
    pub module_name: String,
    pub predictions_made: u32,
    pub predictions_verified_correct: u32,
    pub predictions_verified_incorrect: u32,
    pub accuracy: f32,
    pub confidence_calibration: f32, // How well confidence matches actual accuracy
}

impl Default for EffectivenessTracker {
    fn default() -> Self {
        Self {
            suggestion_stats: HashMap::new(),
            autofix_stats: HashMap::new(),
            module_accuracy: HashMap::new(),
            overall_score: 0.5, // Start neutral
            last_updated: Utc::now(),
        }
    }
}

impl EffectivenessTracker {
    /// Load from disk.
    pub fn load() -> Self {
        let path = Self::storage_path();

        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(tracker) = serde_json::from_str(&contents) {
                return tracker;
            }
        }

        Self::default()
    }

    /// Save to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::storage_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;

        Ok(())
    }

    fn storage_path() -> PathBuf {
        PathBuf::from("/var/lib/anna/effectiveness.json")
    }

    /// Record that a suggestion was made.
    pub fn record_suggestion(&mut self, category: &str) {
        let stats = self.suggestion_stats.entry(category.to_string()).or_insert_with(CategoryStats::new);
        stats.total_attempts += 1;
        self.last_updated = Utc::now();
    }

    /// Record user's response to a suggestion.
    pub fn record_suggestion_feedback(&mut self, category: &str, accepted: bool, worked: Option<bool>) {
        let stats = self.suggestion_stats.entry(category.to_string()).or_insert_with(CategoryStats::new);

        if accepted {
            stats.user_accepted += 1;
            if let Some(true) = worked {
                stats.successful += 1;
            } else if let Some(false) = worked {
                stats.failed += 1;
            }
        } else {
            stats.user_rejected += 1;
        }

        stats.update_rates();
        self.update_overall_score();
        self.last_updated = Utc::now();

        if let Err(e) = self.save() {
            warn!("Failed to save effectiveness tracker: {}", e);
        }
    }

    /// Record an auto-fix attempt.
    pub fn record_autofix_attempt(&mut self, failure_type: &str, success: bool) {
        let stats = self.autofix_stats.entry(failure_type.to_string()).or_insert_with(CategoryStats::new);
        stats.total_attempts += 1;

        if success {
            stats.successful += 1;
        } else {
            stats.failed += 1;
        }

        stats.update_rates();
        let success_rate = stats.success_rate;

        self.update_overall_score();
        self.last_updated = Utc::now();

        info!(
            "Auto-fix {}: {} (success rate: {:.0}%)",
            failure_type,
            if success { "success" } else { "failed" },
            success_rate * 100.0
        );

        if let Err(e) = self.save() {
            warn!("Failed to save effectiveness tracker: {}", e);
        }
    }

    /// Record module prediction verification.
    pub fn record_module_verification(&mut self, module_name: &str, predicted_confidence: f32, was_correct: bool) {
        let accuracy = self
            .module_accuracy
            .entry(module_name.to_string())
            .or_insert_with(|| ModuleAccuracy {
                module_name: module_name.to_string(),
                predictions_made: 0,
                predictions_verified_correct: 0,
                predictions_verified_incorrect: 0,
                accuracy: 0.5,
                confidence_calibration: 1.0,
            });

        accuracy.predictions_made += 1;

        if was_correct {
            accuracy.predictions_verified_correct += 1;
        } else {
            accuracy.predictions_verified_incorrect += 1;
        }

        // Update accuracy
        if accuracy.predictions_made > 0 {
            accuracy.accuracy = accuracy.predictions_verified_correct as f32 / accuracy.predictions_made as f32;
        }

        // Update confidence calibration (how well confidence matches accuracy)
        let predicted = predicted_confidence;
        let actual = accuracy.accuracy;
        accuracy.confidence_calibration = 1.0 - (predicted - actual).abs();

        info!(
            "Module '{}': {:.0}% accurate ({}/{} verified), confidence calibration: {:.0}%",
            module_name,
            accuracy.accuracy * 100.0,
            accuracy.predictions_verified_correct,
            accuracy.predictions_made,
            accuracy.confidence_calibration * 100.0
        );

        self.update_overall_score();
        self.last_updated = Utc::now();

        if let Err(e) = self.save() {
            warn!("Failed to save effectiveness tracker: {}", e);
        }
    }

    /// Update overall effectiveness score.
    fn update_overall_score(&mut self) {
        let mut score_sum = 0.0;
        let mut score_count = 0;

        // Average suggestion acceptance rate
        for stats in self.suggestion_stats.values() {
            if stats.user_accepted + stats.user_rejected > 0 {
                score_sum += stats.acceptance_rate;
                score_count += 1;
            }
        }

        // Average auto-fix success rate
        for stats in self.autofix_stats.values() {
            if stats.total_attempts > 0 {
                score_sum += stats.success_rate;
                score_count += 1;
            }
        }

        // Average module accuracy
        for accuracy in self.module_accuracy.values() {
            if accuracy.predictions_made > 0 {
                score_sum += accuracy.accuracy;
                score_count += 1;
            }
        }

        if score_count > 0 {
            self.overall_score = score_sum / score_count as f32;
        }
    }

    /// Get adjusted confidence for a module based on past performance.
    pub fn get_adjusted_confidence(&self, module_name: &str, stated_confidence: f32) -> f32 {
        if let Some(accuracy) = self.module_accuracy.get(module_name) {
            // Adjust stated confidence based on calibration
            // If module is overconfident (calibration < 1.0), reduce confidence
            // If module is underconfident (rare), increase slightly
            stated_confidence * accuracy.confidence_calibration
        } else {
            // No data yet, use stated confidence
            stated_confidence
        }
    }

    /// Should we be more autonomous based on trust level?
    pub fn should_increase_autonomy(&self, category: &str) -> bool {
        if let Some(stats) = self.autofix_stats.get(category) {
            // High success rate and enough attempts
            stats.success_rate > 0.85 && stats.total_attempts >= 10
        } else {
            false
        }
    }

    /// Should we be more cautious based on recent failures?
    pub fn should_decrease_autonomy(&self, category: &str) -> bool {
        if let Some(stats) = self.autofix_stats.get(category) {
            // Low success rate or high rejection rate
            stats.success_rate < 0.60 || stats.acceptance_rate < 0.40
        } else {
            false
        }
    }

    /// Get recommendation for autonomy adjustment.
    pub fn get_autonomy_recommendation(&self) -> Option<AutonomyRecommendation> {
        // Check if overall score suggests adjustment
        if self.overall_score > 0.85 {
            let high_performing: Vec<_> = self
                .autofix_stats
                .iter()
                .filter(|(_, stats)| stats.success_rate > 0.85 && stats.total_attempts >= 10)
                .map(|(cat, _)| cat.clone())
                .collect();

            if !high_performing.is_empty() {
                return Some(AutonomyRecommendation {
                    action: AutonomyAction::Increase,
                    categories: high_performing,
                    reason: format!(
                        "High success rate ({:.0}%) across {} operations. Consider more autonomous mode.",
                        self.overall_score * 100.0,
                        self.autofix_stats.values().map(|s| s.total_attempts).sum::<u32>()
                    ),
                });
            }
        } else if self.overall_score < 0.60 {
            let low_performing: Vec<_> = self
                .autofix_stats
                .iter()
                .filter(|(_, stats)| stats.success_rate < 0.60)
                .map(|(cat, _)| cat.clone())
                .collect();

            if !low_performing.is_empty() {
                return Some(AutonomyRecommendation {
                    action: AutonomyAction::Decrease,
                    categories: low_performing,
                    reason: format!(
                        "Lower success rate ({:.0}%) detected. Being more cautious.",
                        self.overall_score * 100.0
                    ),
                });
            }
        }

        None
    }
}

/// Autonomy recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyRecommendation {
    pub action: AutonomyAction,
    pub categories: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AutonomyAction {
    Increase,
    Decrease,
    Maintain,
}

/// Generate effectiveness report.
pub fn generate_effectiveness_report() -> String {
    let tracker = EffectivenessTracker::load();

    let mut report = format!("Self-Improvement Report (Overall Score: {:.0}/100)\n\n", tracker.overall_score * 100.0);

    // Auto-fix performance
    if !tracker.autofix_stats.is_empty() {
        report.push_str("Auto-Fix Performance:\n");
        let mut stats: Vec<_> = tracker.autofix_stats.iter().collect();
        stats.sort_by(|a, b| b.1.success_rate.partial_cmp(&a.1.success_rate).unwrap());

        for (category, stats) in stats.iter().take(10) {
            report.push_str(&format!(
                "  {} - {:.0}% success ({}/{} attempts)\n",
                category, stats.success_rate * 100.0, stats.successful, stats.total_attempts
            ));
        }
        report.push('\n');
    }

    // Module accuracy
    if !tracker.module_accuracy.is_empty() {
        report.push_str("Analysis Module Accuracy:\n");
        let mut accuracy: Vec<_> = tracker.module_accuracy.values().collect();
        accuracy.sort_by(|a, b| b.accuracy.partial_cmp(&a.accuracy).unwrap());

        for acc in accuracy.iter().take(10) {
            report.push_str(&format!(
                "  {} - {:.0}% accurate ({}/{} verified), confidence calibration: {:.0}%\n",
                acc.module_name,
                acc.accuracy * 100.0,
                acc.predictions_verified_correct,
                acc.predictions_made,
                acc.confidence_calibration * 100.0
            ));
        }
        report.push('\n');
    }

    // Suggestions
    if !tracker.suggestion_stats.is_empty() {
        report.push_str("Suggestion Acceptance:\n");
        let mut stats: Vec<_> = tracker.suggestion_stats.iter().collect();
        stats.sort_by(|a, b| b.1.acceptance_rate.partial_cmp(&a.1.acceptance_rate).unwrap());

        for (category, stats) in stats.iter().take(10) {
            if stats.user_accepted + stats.user_rejected > 0 {
                report.push_str(&format!(
                    "  {} - {:.0}% accepted ({}/{})\n",
                    category,
                    stats.acceptance_rate * 100.0,
                    stats.user_accepted,
                    stats.user_accepted + stats.user_rejected
                ));
            }
        }
        report.push('\n');
    }

    // Autonomy recommendation
    if let Some(rec) = tracker.get_autonomy_recommendation() {
        report.push_str("Autonomy Recommendation:\n");
        report.push_str(&format!("  Action: {:?}\n", rec.action));
        report.push_str(&format!("  Reason: {}\n", rec.reason));
        if !rec.categories.is_empty() {
            report.push_str(&format!("  Categories: {}\n", rec.categories.join(", ")));
        }
    }

    report
}
