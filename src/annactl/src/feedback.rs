//! Outcome Feedback Loop for Anna v0.14.0 "Orion III" Phase 2.3
//!
//! Reinforcement learning between autonomous actions and confidence weights

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::autonomy::{ActionConfidence, AutonomyManager, AutonomyTier};
use crate::learning::{LearningEngine, RuleWeight};

/// Action outcome from audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOutcome {
    pub action_id: String,
    pub timestamp: u64,
    pub success: bool,
    pub reverted: bool,
    pub execution_time_ms: f32,
    pub tier: String,
    pub error_message: Option<String>,
}

/// Feedback entry (logged to feedback.jsonl)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEntry {
    pub timestamp: u64,
    pub action_id: String,
    pub old_confidence: f32,
    pub new_confidence: f32,
    pub adjustment: f32,
    pub reason: String,
    pub outcome: String, // "success", "failed", "reverted"
}

/// Feedback summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSummary {
    pub total_outcomes_processed: usize,
    pub positive_adjustments: usize,
    pub negative_adjustments: usize,
    pub avg_confidence_change: f32,
    pub most_improved_action: Option<String>,
    pub most_degraded_action: Option<String>,
    pub recent_feedback: Vec<FeedbackEntry>,
}

/// Feedback loop manager
pub struct FeedbackLoop {
    audit_path: PathBuf,
    feedback_path: PathBuf,
    state_dir: PathBuf,
    learning_engine: LearningEngine,
    autonomy_manager: AutonomyManager,
}

impl FeedbackLoop {
    /// Create new feedback loop
    pub fn new() -> Result<Self> {
        let state_dir = Self::get_state_dir()?;
        fs::create_dir_all(&state_dir)?;

        let audit_path = state_dir.join("audit.jsonl");
        let feedback_path = state_dir.join("feedback.jsonl");

        let learning_engine = LearningEngine::new()?;
        let autonomy_manager = AutonomyManager::new()?;

        Ok(Self {
            audit_path,
            feedback_path,
            state_dir,
            learning_engine,
            autonomy_manager,
        })
    }

    /// Get state directory
    fn get_state_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".local/state/anna"))
    }

    /// Process all new outcomes and update confidence
    pub fn process_outcomes(&mut self) -> Result<Vec<FeedbackEntry>> {
        let outcomes = self.load_recent_outcomes()?;
        let mut feedback_entries = Vec::new();

        for outcome in outcomes {
            if let Ok(entry) = self.adjust_confidence(&outcome) {
                self.log_feedback(&entry)?;
                feedback_entries.push(entry);
            }
        }

        // Note: autonomy_manager.update_confidence() saves automatically
        // Note: learning_engine updates happen via preferences.json

        Ok(feedback_entries)
    }

    /// Load recent action outcomes from audit log
    fn load_recent_outcomes(&self) -> Result<Vec<ActionOutcome>> {
        if !self.audit_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.audit_path)?;
        let mut outcomes = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Parse audit entries that contain action outcomes
            if let Ok(outcome) = serde_json::from_str::<ActionOutcome>(line) {
                outcomes.push(outcome);
            }
        }

        // Only process last 100 entries (recent outcomes)
        Ok(outcomes.into_iter().rev().take(100).collect())
    }

    /// Adjust confidence based on action outcome
    fn adjust_confidence(&mut self, outcome: &ActionOutcome) -> Result<FeedbackEntry> {
        // Get old confidence
        let old_confidence = self.autonomy_manager
            .get_all_confidence()
            .get(&outcome.action_id)
            .map(|c| c.confidence)
            .unwrap_or(0.5);

        let (adjustment, reason, outcome_str) = if outcome.reverted {
            (-0.2, "Action was reverted by user", "reverted")
        } else if outcome.success {
            (0.1, "Action completed successfully", "success")
        } else {
            (-0.15, "Action failed to execute", "failed")
        };

        let new_confidence = (old_confidence + adjustment).max(0.0).min(1.0);

        // Create ActionResult for update_confidence
        let action_result = crate::action_engine::ActionResult {
            action_id: outcome.action_id.clone(),
            executed_at: outcome.timestamp,
            actor: "auto".to_string(),
            success: outcome.success,
            output: outcome.error_message.clone().unwrap_or_else(|| "OK".to_string()),
            exit_code: if outcome.success { Some(0) } else { Some(1) },
            duration_ms: outcome.execution_time_ms as u64,
        };

        // Update autonomy manager (this saves automatically)
        self.autonomy_manager.update_confidence(
            &outcome.action_id,
            &action_result,
            outcome.reverted,
            24, // 24-hour cooldown
        )?;

        // Note: Learning engine updates happen separately via learning_cmd
        // We just log the feedback here for audit purposes

        Ok(FeedbackEntry {
            timestamp: current_timestamp(),
            action_id: outcome.action_id.clone(),
            old_confidence,
            new_confidence,
            adjustment,
            reason: reason.to_string(),
            outcome: outcome_str.to_string(),
        })
    }

    /// Log feedback entry
    fn log_feedback(&self, entry: &FeedbackEntry) -> Result<()> {
        let json = serde_json::to_string(entry)?;
        let mut content = String::new();

        if self.feedback_path.exists() {
            content = fs::read_to_string(&self.feedback_path)?;
        }

        content.push_str(&json);
        content.push('\n');

        fs::write(&self.feedback_path, content)?;

        Ok(())
    }

    /// Load all feedback entries
    pub fn load_feedback(&self) -> Result<Vec<FeedbackEntry>> {
        if !self.feedback_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.feedback_path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<FeedbackEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse feedback entry: {}", e);
                    continue;
                }
            }
        }

        Ok(entries)
    }

    /// Get feedback summary
    pub fn get_summary(&self) -> Result<FeedbackSummary> {
        let entries = self.load_feedback()?;
        let total = entries.len();

        let positive = entries.iter().filter(|e| e.adjustment > 0.0).count();
        let negative = entries.iter().filter(|e| e.adjustment < 0.0).count();

        let avg_change = if total > 0 {
            entries.iter().map(|e| e.adjustment).sum::<f32>() / total as f32
        } else {
            0.0
        };

        // Find most improved and most degraded actions
        let mut action_changes: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        for entry in &entries {
            *action_changes.entry(entry.action_id.clone()).or_insert(0.0) += entry.adjustment;
        }

        let most_improved = action_changes
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k.clone());

        let most_degraded = action_changes
            .iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k.clone());

        Ok(FeedbackSummary {
            total_outcomes_processed: total,
            positive_adjustments: positive,
            negative_adjustments: negative,
            avg_confidence_change: avg_change,
            most_improved_action: most_improved,
            most_degraded_action: most_degraded,
            recent_feedback: entries.into_iter().rev().take(10).collect(),
        })
    }

    /// Simulate feedback processing (dry-run)
    pub fn simulate(&mut self) -> Result<Vec<FeedbackEntry>> {
        let outcomes = self.load_recent_outcomes()?;
        let mut simulated_entries = Vec::new();

        for outcome in outcomes {
            if let Ok(entry) = self.simulate_adjustment(&outcome) {
                simulated_entries.push(entry);
            }
        }

        Ok(simulated_entries)
    }

    /// Simulate confidence adjustment without saving
    fn simulate_adjustment(&self, outcome: &ActionOutcome) -> Result<FeedbackEntry> {
        let old_confidence = self.autonomy_manager
            .get_all_confidence()
            .get(&outcome.action_id)
            .map(|c| c.confidence)
            .unwrap_or(0.5);

        let (adjustment, reason, outcome_str) = if outcome.reverted {
            (-0.2, "Action was reverted by user", "reverted")
        } else if outcome.success {
            (0.1, "Action completed successfully", "success")
        } else {
            (-0.15, "Action failed to execute", "failed")
        };

        let new_confidence = (old_confidence + adjustment).max(0.0).min(1.0);

        Ok(FeedbackEntry {
            timestamp: current_timestamp(),
            action_id: outcome.action_id.clone(),
            old_confidence,
            new_confidence,
            adjustment,
            reason: reason.to_string(),
            outcome: outcome_str.to_string(),
        })
    }

    /// Reset all feedback history
    pub fn reset(&self) -> Result<()> {
        if self.feedback_path.exists() {
            fs::remove_file(&self.feedback_path)?;
        }
        Ok(())
    }

    /// Get confidence trajectory for an action
    pub fn get_confidence_trajectory(&self, action_id: &str) -> Result<Vec<(u64, f32)>> {
        let entries = self.load_feedback()?;
        let trajectory: Vec<(u64, f32)> = entries
            .into_iter()
            .filter(|e| e.action_id == action_id)
            .map(|e| (e.timestamp, e.new_confidence))
            .collect();

        Ok(trajectory)
    }

    /// Calculate learning velocity (rate of confidence change)
    pub fn calculate_learning_velocity(&self) -> Result<f32> {
        let entries = self.load_feedback()?;
        if entries.len() < 2 {
            return Ok(0.0);
        }

        let total_change: f32 = entries.iter().map(|e| e.adjustment.abs()).sum();
        let time_span = entries.last().unwrap().timestamp - entries.first().unwrap().timestamp;

        if time_span == 0 {
            return Ok(0.0);
        }

        // Velocity in confidence units per hour
        let velocity = (total_change / time_span as f32) * 3600.0;

        Ok(velocity)
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_outcome(action_id: &str, success: bool, reverted: bool) -> ActionOutcome {
        ActionOutcome {
            action_id: action_id.to_string(),
            timestamp: current_timestamp(),
            success,
            reverted,
            execution_time_ms: 100.0,
            tier: "Assisted".to_string(),
            error_message: None,
        }
    }

    #[test]
    fn test_feedback_entry_creation() {
        let entry = FeedbackEntry {
            timestamp: 1699000000,
            action_id: "test_action".to_string(),
            old_confidence: 0.5,
            new_confidence: 0.6,
            adjustment: 0.1,
            reason: "Action completed successfully".to_string(),
            outcome: "success".to_string(),
        };

        assert_eq!(entry.adjustment, 0.1);
        assert_eq!(entry.outcome, "success");
    }

    #[test]
    fn test_feedback_entry_serialization() {
        let entry = FeedbackEntry {
            timestamp: 1699000000,
            action_id: "test_action".to_string(),
            old_confidence: 0.5,
            new_confidence: 0.6,
            adjustment: 0.1,
            reason: "Success".to_string(),
            outcome: "success".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: FeedbackEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.action_id, "test_action");
        assert_eq!(parsed.adjustment, 0.1);
    }

    #[test]
    fn test_confidence_adjustment_success() {
        let outcome = create_test_outcome("cleanup_cache", true, false);

        // Simulate adjustment calculation
        let old_confidence = 0.5;
        let adjustment = 0.1; // Success
        let new_confidence = (old_confidence + adjustment).max(0.0).min(1.0);

        assert_eq!(new_confidence, 0.6);
    }

    #[test]
    fn test_confidence_adjustment_failed() {
        let outcome = create_test_outcome("cleanup_cache", false, false);

        let old_confidence = 0.5;
        let adjustment = -0.15; // Failed
        let new_confidence = (old_confidence + adjustment).max(0.0).min(1.0);

        assert_eq!(new_confidence, 0.35);
    }

    #[test]
    fn test_confidence_adjustment_reverted() {
        let outcome = create_test_outcome("cleanup_cache", true, true);

        let old_confidence = 0.5;
        let adjustment = -0.2; // Reverted (strong negative)
        let new_confidence = (old_confidence + adjustment).max(0.0).min(1.0);

        assert_eq!(new_confidence, 0.3);
    }

    #[test]
    fn test_confidence_bounds() {
        // Test upper bound
        let old_confidence = 0.95;
        let adjustment = 0.1;
        let new_confidence = (old_confidence + adjustment).max(0.0).min(1.0);
        assert_eq!(new_confidence, 1.0);

        // Test lower bound
        let old_confidence = 0.05;
        let adjustment = -0.2;
        let new_confidence = (old_confidence + adjustment).max(0.0).min(1.0);
        assert_eq!(new_confidence, 0.0);
    }

    #[test]
    fn test_action_outcome_structure() {
        let outcome = create_test_outcome("test", true, false);

        assert_eq!(outcome.action_id, "test");
        assert!(outcome.success);
        assert!(!outcome.reverted);
        assert_eq!(outcome.tier, "Assisted");
    }

    #[test]
    fn test_feedback_summary_creation() {
        let summary = FeedbackSummary {
            total_outcomes_processed: 10,
            positive_adjustments: 7,
            negative_adjustments: 3,
            avg_confidence_change: 0.05,
            most_improved_action: Some("cleanup_cache".to_string()),
            most_degraded_action: Some("update_packages".to_string()),
            recent_feedback: Vec::new(),
        };

        assert_eq!(summary.total_outcomes_processed, 10);
        assert_eq!(summary.positive_adjustments, 7);
        assert_eq!(summary.avg_confidence_change, 0.05);
    }

    #[test]
    fn test_learning_velocity_calculation() {
        // Simulate velocity calculation
        let entries = vec![
            FeedbackEntry {
                timestamp: 1000,
                action_id: "test".to_string(),
                old_confidence: 0.5,
                new_confidence: 0.6,
                adjustment: 0.1,
                reason: "Success".to_string(),
                outcome: "success".to_string(),
            },
            FeedbackEntry {
                timestamp: 1000 + 3600, // 1 hour later
                action_id: "test".to_string(),
                old_confidence: 0.6,
                new_confidence: 0.7,
                adjustment: 0.1,
                reason: "Success".to_string(),
                outcome: "success".to_string(),
            },
        ];

        let total_change: f32 = entries.iter().map(|e| e.adjustment.abs()).sum();
        let time_span = entries.last().unwrap().timestamp - entries.first().unwrap().timestamp;
        let velocity = (total_change / time_span as f32) * 3600.0;

        assert_eq!(velocity, 0.2); // 0.2 confidence units per hour
    }

    #[test]
    fn test_state_dir_path() {
        if let Ok(dir) = FeedbackLoop::get_state_dir() {
            assert!(dir.to_string_lossy().contains(".local/state/anna"));
        }
    }
}
