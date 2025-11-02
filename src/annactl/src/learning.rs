//! Behavior Learning System for Anna v0.14.0 "Orion III" Phase 2.2
//!
//! Makes Anna adaptive by learning from human interaction patterns.
//! Tracks which advice is accepted, ignored, or reverted, and adjusts priorities autonomously.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::audit_log::{AuditEntry, AuditLog};

/// Learned behavior weights for a specific rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleWeight {
    pub rule_id: String,
    pub user_response_weight: f32,  // -1.0 to 1.0, 0 is neutral
    pub auto_confidence: f32,        // 0.0 to 1.0, confidence in auto-running
    pub total_shown: u32,            // Times recommendation was shown
    pub accepted: u32,               // Times user followed advice
    pub ignored: u32,                // Times user did not follow
    pub reverted: u32,               // Times user undid the action
    pub auto_runs: u32,              // Times auto-executed successfully
    pub last_updated: u64,           // Timestamp of last weight update
}

impl RuleWeight {
    /// Create new weight entry for a rule
    pub fn new(rule_id: String) -> Self {
        Self {
            rule_id,
            user_response_weight: 0.0,
            auto_confidence: 0.5,  // Start neutral
            total_shown: 0,
            accepted: 0,
            ignored: 0,
            reverted: 0,
            auto_runs: 0,
            last_updated: current_timestamp(),
        }
    }

    /// Calculate acceptance rate
    pub fn acceptance_rate(&self) -> f32 {
        if self.total_shown == 0 {
            return 0.5;  // Neutral if no data
        }
        self.accepted as f32 / self.total_shown as f32
    }

    /// Calculate ignore rate
    pub fn ignore_rate(&self) -> f32 {
        if self.total_shown == 0 {
            return 0.0;
        }
        self.ignored as f32 / self.total_shown as f32
    }

    /// Calculate revert rate
    pub fn revert_rate(&self) -> f32 {
        if self.accepted == 0 {
            return 0.0;
        }
        self.reverted as f32 / self.accepted as f32
    }

    /// Update weights based on new interaction
    pub fn update(&mut self, interaction: &Interaction) {
        match interaction.response {
            UserResponse::Accepted => {
                self.total_shown += 1;
                self.accepted += 1;
                // Increase confidence
                self.user_response_weight = (self.user_response_weight + 0.1).min(1.0);
                self.auto_confidence = (self.auto_confidence + 0.05).min(1.0);
            }
            UserResponse::Ignored => {
                self.total_shown += 1;
                self.ignored += 1;
                // Decrease confidence
                self.user_response_weight = (self.user_response_weight - 0.15).max(-1.0);
                self.auto_confidence = (self.auto_confidence - 0.1).max(0.0);
            }
            UserResponse::Reverted => {
                self.reverted += 1;
                // Strong negative signal
                self.user_response_weight = (self.user_response_weight - 0.3).max(-1.0);
                self.auto_confidence = (self.auto_confidence - 0.2).max(0.0);
            }
            UserResponse::AutoRan => {
                self.auto_runs += 1;
                // Slight increase if auto-run succeeded
                self.auto_confidence = (self.auto_confidence + 0.02).min(1.0);
            }
        }

        self.last_updated = current_timestamp();
    }

    /// Get trust level as string
    pub fn trust_level(&self) -> &'static str {
        if self.revert_rate() > 0.3 {
            "untrusted"
        } else if self.user_response_weight < -0.5 {
            "low"
        } else if self.user_response_weight > 0.5 {
            "high"
        } else {
            "neutral"
        }
    }
}

/// User response to a recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserResponse {
    Accepted,  // User followed the advice
    Ignored,   // User did not follow the advice
    Reverted,  // User undid the action
    AutoRan,   // Action auto-executed successfully
}

/// Parsed interaction from audit log
#[derive(Debug, Clone)]
pub struct Interaction {
    pub rule_id: String,
    pub response: UserResponse,
    pub timestamp: u64,
}

/// Behavior learning engine
pub struct LearningEngine {
    weights: HashMap<String, RuleWeight>,
    preferences_path: PathBuf,
    audit_log: AuditLog,
}

impl LearningEngine {
    /// Create new learning engine
    pub fn new() -> Result<Self> {
        let state_dir = Self::get_state_dir()?;
        fs::create_dir_all(&state_dir)?;

        let preferences_path = state_dir.join("preferences.json");
        let audit_log = AuditLog::new()?;

        let mut engine = Self {
            weights: HashMap::new(),
            preferences_path,
            audit_log,
        };

        // Load existing preferences
        if let Ok(loaded) = engine.load_preferences() {
            engine.weights = loaded;
        }

        Ok(engine)
    }

    /// Get state directory
    fn get_state_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".local/state/anna"))
    }

    /// Load preferences from disk
    fn load_preferences(&self) -> Result<HashMap<String, RuleWeight>> {
        if !self.preferences_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.preferences_path)?;
        let weights: HashMap<String, RuleWeight> = serde_json::from_str(&content)?;
        Ok(weights)
    }

    /// Save preferences to disk
    pub fn save_preferences(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.weights)?;
        fs::write(&self.preferences_path, json)?;
        Ok(())
    }

    /// Parse audit log and update weights
    pub fn learn_from_audit(&mut self) -> Result<LearningSummary> {
        let entries = self.audit_log.load_all()?;
        let interactions = Self::parse_interactions(&entries)?;

        let mut new_interactions = 0;
        let mut rules_updated = 0;

        for interaction in interactions {
            let weight = self.weights
                .entry(interaction.rule_id.clone())
                .or_insert_with(|| RuleWeight::new(interaction.rule_id.clone()));

            let old_trust = weight.trust_level();
            weight.update(&interaction);
            let new_trust = weight.trust_level();

            if old_trust != new_trust {
                rules_updated += 1;
            }

            new_interactions += 1;
        }

        self.save_preferences()?;

        Ok(LearningSummary {
            total_rules: self.weights.len(),
            new_interactions,
            rules_updated,
            high_confidence: self.weights.values().filter(|w| w.trust_level() == "high").count(),
            low_confidence: self.weights.values().filter(|w| w.trust_level() == "low").count(),
            untrusted: self.weights.values().filter(|w| w.trust_level() == "untrusted").count(),
        })
    }

    /// Parse interactions from audit entries
    fn parse_interactions(entries: &[AuditEntry]) -> Result<Vec<Interaction>> {
        let mut interactions = Vec::new();

        for entry in entries {
            // Look for advisor recommendations
            if entry.actor == "advisor" && entry.action_type == "recommend" {
                // Check if followed by user action
                if let Some(action_id) = &entry.related_action_id {
                    let response = match entry.result.as_str() {
                        "success" => UserResponse::Accepted,
                        "ignored" => UserResponse::Ignored,
                        _ => continue,
                    };

                    interactions.push(Interaction {
                        rule_id: action_id.clone(),
                        response,
                        timestamp: entry.timestamp,
                    });
                }
            }

            // Look for reverts
            if entry.action_type == "revert" {
                if let Some(action_id) = &entry.related_action_id {
                    interactions.push(Interaction {
                        rule_id: action_id.clone(),
                        response: UserResponse::Reverted,
                        timestamp: entry.timestamp,
                    });
                }
            }

            // Look for auto-runs
            if entry.actor == "auto" && entry.result == "success" {
                if let Some(action_id) = &entry.related_action_id {
                    interactions.push(Interaction {
                        rule_id: action_id.clone(),
                        response: UserResponse::AutoRan,
                        timestamp: entry.timestamp,
                    });
                }
            }
        }

        Ok(interactions)
    }

    /// Get weight for a specific rule
    pub fn get_weight(&self, rule_id: &str) -> Option<&RuleWeight> {
        self.weights.get(rule_id)
    }

    /// Get all weights
    pub fn get_all_weights(&self) -> &HashMap<String, RuleWeight> {
        &self.weights
    }

    /// Reset all learned weights
    pub fn reset(&mut self) -> Result<()> {
        self.weights.clear();
        self.save_preferences()?;
        Ok(())
    }

    /// Get behavioral trend analysis
    pub fn get_trend(&self) -> BehavioralTrend {
        let total_rules = self.weights.len();

        if total_rules == 0 {
            return BehavioralTrend {
                overall_trust: 0.5,
                acceptance_trend: 0.0,
                automation_readiness: 0.0,
                top_accepted: Vec::new(),
                top_ignored: Vec::new(),
                untrusted_rules: Vec::new(),
            };
        }

        // Calculate overall metrics
        let total_weight: f32 = self.weights.values().map(|w| w.user_response_weight).sum();
        let overall_trust = (total_weight / total_rules as f32 + 1.0) / 2.0; // Normalize to 0-1

        let total_acceptance: f32 = self.weights.values().map(|w| w.acceptance_rate()).sum();
        let acceptance_trend = total_acceptance / total_rules as f32;

        let total_auto_confidence: f32 = self.weights.values().map(|w| w.auto_confidence).sum();
        let automation_readiness = total_auto_confidence / total_rules as f32;

        // Get top accepted rules
        let mut by_acceptance: Vec<_> = self.weights.values().collect();
        by_acceptance.sort_by(|a, b| b.acceptance_rate().partial_cmp(&a.acceptance_rate()).unwrap());
        let top_accepted: Vec<String> = by_acceptance.iter()
            .take(5)
            .map(|w| w.rule_id.clone())
            .collect();

        // Get top ignored rules
        let mut by_ignore: Vec<_> = self.weights.values().collect();
        by_ignore.sort_by(|a, b| b.ignore_rate().partial_cmp(&a.ignore_rate()).unwrap());
        let top_ignored: Vec<String> = by_ignore.iter()
            .take(5)
            .map(|w| w.rule_id.clone())
            .collect();

        // Get untrusted rules
        let untrusted_rules: Vec<String> = self.weights.values()
            .filter(|w| w.trust_level() == "untrusted")
            .map(|w| w.rule_id.clone())
            .collect();

        BehavioralTrend {
            overall_trust,
            acceptance_trend,
            automation_readiness,
            top_accepted,
            top_ignored,
            untrusted_rules,
        }
    }
}

/// Learning summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSummary {
    pub total_rules: usize,
    pub new_interactions: usize,
    pub rules_updated: usize,
    pub high_confidence: usize,
    pub low_confidence: usize,
    pub untrusted: usize,
}

/// Behavioral trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralTrend {
    pub overall_trust: f32,          // 0.0 to 1.0
    pub acceptance_trend: f32,       // 0.0 to 1.0
    pub automation_readiness: f32,   // 0.0 to 1.0
    pub top_accepted: Vec<String>,
    pub top_ignored: Vec<String>,
    pub untrusted_rules: Vec<String>,
}

/// Get current timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_weight_creation() {
        let weight = RuleWeight::new("test_rule".to_string());
        assert_eq!(weight.rule_id, "test_rule");
        assert_eq!(weight.user_response_weight, 0.0);
        assert_eq!(weight.auto_confidence, 0.5);
        assert_eq!(weight.total_shown, 0);
    }

    #[test]
    fn test_acceptance_rate() {
        let mut weight = RuleWeight::new("test".to_string());
        assert_eq!(weight.acceptance_rate(), 0.5); // No data = neutral

        weight.total_shown = 10;
        weight.accepted = 7;
        assert_eq!(weight.acceptance_rate(), 0.7);
    }

    #[test]
    fn test_weight_update_accepted() {
        let mut weight = RuleWeight::new("test".to_string());
        let interaction = Interaction {
            rule_id: "test".to_string(),
            response: UserResponse::Accepted,
            timestamp: 12345,
        };

        weight.update(&interaction);
        assert_eq!(weight.total_shown, 1);
        assert_eq!(weight.accepted, 1);
        assert!(weight.user_response_weight > 0.0);
        assert!(weight.auto_confidence > 0.5);
    }

    #[test]
    fn test_weight_update_ignored() {
        let mut weight = RuleWeight::new("test".to_string());
        let interaction = Interaction {
            rule_id: "test".to_string(),
            response: UserResponse::Ignored,
            timestamp: 12345,
        };

        weight.update(&interaction);
        assert_eq!(weight.total_shown, 1);
        assert_eq!(weight.ignored, 1);
        assert!(weight.user_response_weight < 0.0);
        assert!(weight.auto_confidence < 0.5);
    }

    #[test]
    fn test_weight_update_reverted() {
        let mut weight = RuleWeight::new("test".to_string());
        let interaction = Interaction {
            rule_id: "test".to_string(),
            response: UserResponse::Reverted,
            timestamp: 12345,
        };

        weight.update(&interaction);
        assert_eq!(weight.reverted, 1);
        assert!(weight.user_response_weight < -0.2);
        assert!(weight.auto_confidence < 0.3);
    }

    #[test]
    fn test_trust_level_calculation() {
        let mut weight = RuleWeight::new("test".to_string());
        assert_eq!(weight.trust_level(), "neutral");

        // High trust
        weight.user_response_weight = 0.7;
        assert_eq!(weight.trust_level(), "high");

        // Low trust
        weight.user_response_weight = -0.7;
        assert_eq!(weight.trust_level(), "low");

        // Untrusted (high revert rate)
        weight.accepted = 10;
        weight.reverted = 4;
        assert_eq!(weight.trust_level(), "untrusted");
    }

    #[test]
    fn test_multiple_interactions() {
        let mut weight = RuleWeight::new("test".to_string());

        // 7 accepts
        for _ in 0..7 {
            weight.update(&Interaction {
                rule_id: "test".to_string(),
                response: UserResponse::Accepted,
                timestamp: 12345,
            });
        }

        // 3 ignores
        for _ in 0..3 {
            weight.update(&Interaction {
                rule_id: "test".to_string(),
                response: UserResponse::Ignored,
                timestamp: 12345,
            });
        }

        assert_eq!(weight.total_shown, 10);
        assert_eq!(weight.accepted, 7);
        assert_eq!(weight.ignored, 3);
        assert_eq!(weight.acceptance_rate(), 0.7);
    }

    #[test]
    fn test_behavioral_trend_empty() {
        let engine = LearningEngine {
            weights: HashMap::new(),
            preferences_path: PathBuf::from("/tmp/test_preferences.json"),
            audit_log: AuditLog::new().unwrap(),
        };

        let trend = engine.get_trend();
        assert_eq!(trend.overall_trust, 0.5);
        assert_eq!(trend.acceptance_trend, 0.0);
        assert_eq!(trend.automation_readiness, 0.0);
    }

    #[test]
    fn test_behavioral_trend_with_data() {
        let mut weights = HashMap::new();

        let mut w1 = RuleWeight::new("rule1".to_string());
        w1.user_response_weight = 0.8;
        w1.auto_confidence = 0.9;
        w1.total_shown = 10;
        w1.accepted = 9;
        weights.insert("rule1".to_string(), w1);

        let mut w2 = RuleWeight::new("rule2".to_string());
        w2.user_response_weight = -0.5;
        w2.auto_confidence = 0.2;
        w2.total_shown = 10;
        w2.ignored = 8;
        weights.insert("rule2".to_string(), w2);

        let engine = LearningEngine {
            weights,
            preferences_path: PathBuf::from("/tmp/test_preferences.json"),
            audit_log: AuditLog::new().unwrap(),
        };

        let trend = engine.get_trend();
        assert!(trend.overall_trust > 0.4 && trend.overall_trust < 0.6);
        assert!(trend.acceptance_trend > 0.3);
    }

    #[test]
    fn test_state_dir_path() {
        if let Ok(dir) = LearningEngine::get_state_dir() {
            assert!(dir.to_string_lossy().contains(".local/state/anna"));
        }
    }
}
