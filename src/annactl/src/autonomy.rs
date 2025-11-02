//! Autonomy System for Anna v0.14.0 "Orion III" Phase 2.3
//!
//! Confidence-based autonomous action execution with escalation and demotion

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::action_engine::{Action, ActionResult};

/// Autonomy tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomyTier {
    Observer,    // Analyze and recommend only
    Assisted,    // Execute medium-confidence with confirmation
    Autonomous,  // Auto-run safe, high-confidence actions
}

impl AutonomyTier {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Observer => "Observer Mode",
            Self::Assisted => "Assisted Mode",
            Self::Autonomous => "Autonomous Mode",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Observer => "Analyze and recommend only. No autonomous execution.",
            Self::Assisted => "Execute medium-confidence actions with confirmation.",
            Self::Autonomous => "Auto-run safe, high-confidence actions without confirmation.",
        }
    }

    /// Get emoji
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Observer => "👁️",
            Self::Assisted => "🤝",
            Self::Autonomous => "🤖",
        }
    }

    /// Get numeric level (for comparison)
    pub fn level(&self) -> u8 {
        match self {
            Self::Observer => 0,
            Self::Assisted => 1,
            Self::Autonomous => 2,
        }
    }
}

/// Action confidence tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfidence {
    pub action_id: String,
    pub confidence: f32,          // 0.0 to 1.0
    pub total_runs: u32,
    pub successful_runs: u32,
    pub failed_runs: u32,
    pub reverted_runs: u32,
    pub last_run: Option<u64>,    // Timestamp of last execution
    pub next_allowed_run: u64,    // Cooldown expiry timestamp
    pub last_updated: u64,
}

impl ActionConfidence {
    /// Create new confidence tracker
    pub fn new(action_id: String) -> Self {
        Self {
            action_id,
            confidence: 0.5,  // Start neutral
            total_runs: 0,
            successful_runs: 0,
            failed_runs: 0,
            reverted_runs: 0,
            last_run: None,
            next_allowed_run: 0,
            last_updated: current_timestamp(),
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        if self.total_runs == 0 {
            return 0.0;
        }
        self.successful_runs as f32 / self.total_runs as f32
    }

    /// Calculate failure rate
    pub fn failure_rate(&self) -> f32 {
        if self.total_runs == 0 {
            return 0.0;
        }
        self.failed_runs as f32 / self.total_runs as f32
    }

    /// Calculate revert rate
    pub fn revert_rate(&self) -> f32 {
        if self.successful_runs == 0 {
            return 0.0;
        }
        self.reverted_runs as f32 / self.successful_runs as f32
    }

    /// Update confidence after action execution
    pub fn update_from_result(&mut self, result: &ActionResult, reverted: bool, cooldown_hours: u64) {
        self.total_runs += 1;
        self.last_run = Some(result.executed_at);

        if reverted {
            self.reverted_runs += 1;
            // Strong negative signal
            self.confidence = (self.confidence - 0.2).max(0.0);
        } else if result.success {
            self.successful_runs += 1;
            // Positive signal
            self.confidence = (self.confidence + 0.1).min(1.0);
        } else {
            self.failed_runs += 1;
            // Negative signal
            self.confidence = (self.confidence - 0.15).max(0.0);
        }

        // Set cooldown
        self.next_allowed_run = current_timestamp() + (cooldown_hours * 3600);
        self.last_updated = current_timestamp();
    }

    /// Check if action is in cooldown
    pub fn is_in_cooldown(&self) -> bool {
        current_timestamp() < self.next_allowed_run
    }

    /// Get confidence tier
    pub fn tier(&self) -> ConfidenceTier {
        if self.confidence >= 0.8 {
            ConfidenceTier::High
        } else if self.confidence >= 0.5 {
            ConfidenceTier::Medium
        } else {
            ConfidenceTier::Low
        }
    }

    /// Check if ready for escalation (90% success over 5 runs)
    pub fn ready_for_escalation(&self) -> bool {
        self.total_runs >= 5 && self.success_rate() > 0.9
    }

    /// Check if needs demotion (20% failure rate)
    pub fn needs_demotion(&self) -> bool {
        self.total_runs >= 3 && self.failure_rate() > 0.2
    }
}

/// Confidence tier for execution policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceTier {
    High,    // ≥ 0.8 confidence
    Medium,  // 0.5-0.8 confidence
    Low,     // < 0.5 confidence
}

impl ConfidenceTier {
    /// Get emoji
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::High => "✅",
            Self::Medium => "⚠️",
            Self::Low => "❌",
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }
}

/// Execution decision based on autonomy tier and confidence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionDecision {
    AutoRun,       // Execute automatically
    Confirm,       // Require confirmation
    LogOnly,       // Log only, don't execute
    Cooldown,      // In cooldown period
}

impl ExecutionDecision {
    /// Determine execution decision
    pub fn determine(
        tier: AutonomyTier,
        confidence_tier: ConfidenceTier,
        in_cooldown: bool,
    ) -> Self {
        if in_cooldown {
            return Self::Cooldown;
        }

        match tier {
            AutonomyTier::Observer => Self::LogOnly,
            AutonomyTier::Assisted => match confidence_tier {
                ConfidenceTier::High | ConfidenceTier::Medium => Self::Confirm,
                ConfidenceTier::Low => Self::LogOnly,
            },
            AutonomyTier::Autonomous => match confidence_tier {
                ConfidenceTier::High => Self::AutoRun,
                ConfidenceTier::Medium => Self::Confirm,
                ConfidenceTier::Low => Self::LogOnly,
            },
        }
    }

    /// Get emoji
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::AutoRun => "🤖",
            Self::Confirm => "🤝",
            Self::LogOnly => "📋",
            Self::Cooldown => "⏰",
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::AutoRun => "Auto-Run",
            Self::Confirm => "Confirmation Required",
            Self::LogOnly => "Log Only",
            Self::Cooldown => "In Cooldown",
        }
    }
}

/// Autonomy manager
pub struct AutonomyManager {
    tier: AutonomyTier,
    confidence_map: HashMap<String, ActionConfidence>,
    config_path: PathBuf,
    confidence_path: PathBuf,
}

impl AutonomyManager {
    /// Create new autonomy manager
    pub fn new() -> Result<Self> {
        let state_dir = Self::get_state_dir()?;
        fs::create_dir_all(&state_dir)?;

        let config_path = state_dir.join("autonomy.json");
        let confidence_path = state_dir.join("action_confidence.json");

        // Load tier from disk
        let tier = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or(AutonomyTier::Observer)
        } else {
            AutonomyTier::Observer  // Start in Observer mode
        };

        // Load confidence map
        let confidence_map = if confidence_path.exists() {
            let content = fs::read_to_string(&confidence_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            tier,
            confidence_map,
            config_path,
            confidence_path,
        })
    }

    /// Get state directory
    fn get_state_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".local/state/anna"))
    }

    /// Get current autonomy tier
    pub fn get_tier(&self) -> AutonomyTier {
        self.tier
    }

    /// Set autonomy tier
    pub fn set_tier(&mut self, tier: AutonomyTier) -> Result<()> {
        self.tier = tier;
        self.save_config()?;
        Ok(())
    }

    /// Promote autonomy tier
    pub fn promote(&mut self) -> Result<AutonomyTier> {
        self.tier = match self.tier {
            AutonomyTier::Observer => AutonomyTier::Assisted,
            AutonomyTier::Assisted => AutonomyTier::Autonomous,
            AutonomyTier::Autonomous => AutonomyTier::Autonomous, // Already max
        };
        self.save_config()?;
        Ok(self.tier)
    }

    /// Demote autonomy tier
    pub fn demote(&mut self) -> Result<AutonomyTier> {
        self.tier = match self.tier {
            AutonomyTier::Observer => AutonomyTier::Observer, // Already min
            AutonomyTier::Assisted => AutonomyTier::Observer,
            AutonomyTier::Autonomous => AutonomyTier::Assisted,
        };
        self.save_config()?;
        Ok(self.tier)
    }

    /// Save config to disk
    fn save_config(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.tier)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// Save confidence map to disk
    fn save_confidence(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.confidence_map)?;
        fs::write(&self.confidence_path, json)?;
        Ok(())
    }

    /// Get confidence for action
    pub fn get_confidence(&mut self, action_id: &str) -> &mut ActionConfidence {
        self.confidence_map
            .entry(action_id.to_string())
            .or_insert_with(|| ActionConfidence::new(action_id.to_string()))
    }

    /// Update confidence from result
    pub fn update_confidence(
        &mut self,
        action_id: &str,
        result: &ActionResult,
        reverted: bool,
        cooldown_hours: u64,
    ) -> Result<()> {
        let confidence = self.get_confidence(action_id);
        confidence.update_from_result(result, reverted, cooldown_hours);
        self.save_confidence()?;
        Ok(())
    }

    /// Get execution decision for action
    pub fn get_execution_decision(&mut self, action: &Action) -> ExecutionDecision {
        let confidence = self.get_confidence(&action.id);
        let confidence_tier = confidence.tier();
        let in_cooldown = confidence.is_in_cooldown();

        ExecutionDecision::determine(self.tier, confidence_tier, in_cooldown)
    }

    /// Get all confidence records
    pub fn get_all_confidence(&self) -> &HashMap<String, ActionConfidence> {
        &self.confidence_map
    }

    /// Check if system health meets promotion criteria
    pub fn check_promotion_criteria(
        &self,
        overall_health: u8,
        critical_anomalies: bool,
        perf_drift: f32,
    ) -> bool {
        overall_health >= 8 && !critical_anomalies && perf_drift < 10.0
    }

    /// Auto-escalate if criteria met
    pub fn auto_escalate_if_ready(
        &mut self,
        overall_health: u8,
        critical_anomalies: bool,
        perf_drift: f32,
    ) -> Result<Option<AutonomyTier>> {
        if self.tier == AutonomyTier::Autonomous {
            return Ok(None); // Already max
        }

        // Check promotion criteria
        if !self.check_promotion_criteria(overall_health, critical_anomalies, perf_drift) {
            return Ok(None);
        }

        // Check if enough actions have proven successful
        let high_confidence_count = self
            .confidence_map
            .values()
            .filter(|c| c.ready_for_escalation())
            .count();

        if high_confidence_count >= 3 {
            let new_tier = self.promote()?;
            Ok(Some(new_tier))
        } else {
            Ok(None)
        }
    }

    /// Auto-demote if failures detected
    pub fn auto_demote_if_needed(&mut self) -> Result<Option<AutonomyTier>> {
        if self.tier == AutonomyTier::Observer {
            return Ok(None); // Already min
        }

        // Check if too many actions need demotion
        let low_confidence_count = self
            .confidence_map
            .values()
            .filter(|c| c.needs_demotion())
            .count();

        if low_confidence_count >= 2 {
            let new_tier = self.demote()?;
            Ok(Some(new_tier))
        } else {
            Ok(None)
        }
    }

    /// Get autonomy summary
    pub fn get_summary(&self) -> AutonomySummary {
        let total_actions = self.confidence_map.len();
        let high_confidence = self.confidence_map.values().filter(|c| c.tier() == ConfidenceTier::High).count();
        let medium_confidence = self.confidence_map.values().filter(|c| c.tier() == ConfidenceTier::Medium).count();
        let low_confidence = self.confidence_map.values().filter(|c| c.tier() == ConfidenceTier::Low).count();

        let total_runs: u32 = self.confidence_map.values().map(|c| c.total_runs).sum();
        let total_successful: u32 = self.confidence_map.values().map(|c| c.successful_runs).sum();
        let total_failed: u32 = self.confidence_map.values().map(|c| c.failed_runs).sum();

        let overall_success_rate = if total_runs > 0 {
            total_successful as f32 / total_runs as f32
        } else {
            0.0
        };

        AutonomySummary {
            current_tier: self.tier,
            total_actions,
            high_confidence,
            medium_confidence,
            low_confidence,
            total_runs,
            overall_success_rate,
            ready_for_promotion: self.confidence_map.values().filter(|c| c.ready_for_escalation()).count(),
            needs_demotion: self.confidence_map.values().filter(|c| c.needs_demotion()).count(),
        }
    }
}

/// Autonomy summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomySummary {
    pub current_tier: AutonomyTier,
    pub total_actions: usize,
    pub high_confidence: usize,
    pub medium_confidence: usize,
    pub low_confidence: usize,
    pub total_runs: u32,
    pub overall_success_rate: f32,
    pub ready_for_promotion: usize,
    pub needs_demotion: usize,
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

    #[test]
    fn test_autonomy_tier_levels() {
        assert_eq!(AutonomyTier::Observer.level(), 0);
        assert_eq!(AutonomyTier::Assisted.level(), 1);
        assert_eq!(AutonomyTier::Autonomous.level(), 2);
    }

    #[test]
    fn test_action_confidence_creation() {
        let confidence = ActionConfidence::new("test_action".to_string());
        assert_eq!(confidence.confidence, 0.5);
        assert_eq!(confidence.total_runs, 0);
        assert_eq!(confidence.success_rate(), 0.0);
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut confidence = ActionConfidence::new("test".to_string());
        confidence.total_runs = 10;
        confidence.successful_runs = 8;
        assert_eq!(confidence.success_rate(), 0.8);
    }

    #[test]
    fn test_confidence_tier_classification() {
        let mut confidence = ActionConfidence::new("test".to_string());

        confidence.confidence = 0.9;
        assert_eq!(confidence.tier(), ConfidenceTier::High);

        confidence.confidence = 0.6;
        assert_eq!(confidence.tier(), ConfidenceTier::Medium);

        confidence.confidence = 0.3;
        assert_eq!(confidence.tier(), ConfidenceTier::Low);
    }

    #[test]
    fn test_ready_for_escalation() {
        let mut confidence = ActionConfidence::new("test".to_string());

        // Not enough runs
        confidence.total_runs = 3;
        confidence.successful_runs = 3;
        assert!(!confidence.ready_for_escalation());

        // Enough runs, high success
        confidence.total_runs = 10;
        confidence.successful_runs = 10;
        assert!(confidence.ready_for_escalation());

        // Enough runs, but lower success
        confidence.successful_runs = 8;
        assert!(!confidence.ready_for_escalation());
    }

    #[test]
    fn test_needs_demotion() {
        let mut confidence = ActionConfidence::new("test".to_string());

        confidence.total_runs = 5;
        confidence.failed_runs = 2;
        assert!(confidence.needs_demotion());

        confidence.failed_runs = 0;
        assert!(!confidence.needs_demotion());
    }

    #[test]
    fn test_execution_decision_observer() {
        let decision = ExecutionDecision::determine(
            AutonomyTier::Observer,
            ConfidenceTier::High,
            false,
        );
        assert_eq!(decision, ExecutionDecision::LogOnly);
    }

    #[test]
    fn test_execution_decision_assisted() {
        let decision_high = ExecutionDecision::determine(
            AutonomyTier::Assisted,
            ConfidenceTier::High,
            false,
        );
        assert_eq!(decision_high, ExecutionDecision::Confirm);

        let decision_low = ExecutionDecision::determine(
            AutonomyTier::Assisted,
            ConfidenceTier::Low,
            false,
        );
        assert_eq!(decision_low, ExecutionDecision::LogOnly);
    }

    #[test]
    fn test_execution_decision_autonomous() {
        let decision_high = ExecutionDecision::determine(
            AutonomyTier::Autonomous,
            ConfidenceTier::High,
            false,
        );
        assert_eq!(decision_high, ExecutionDecision::AutoRun);

        let decision_medium = ExecutionDecision::determine(
            AutonomyTier::Autonomous,
            ConfidenceTier::Medium,
            false,
        );
        assert_eq!(decision_medium, ExecutionDecision::Confirm);

        let decision_low = ExecutionDecision::determine(
            AutonomyTier::Autonomous,
            ConfidenceTier::Low,
            false,
        );
        assert_eq!(decision_low, ExecutionDecision::LogOnly);
    }

    #[test]
    fn test_cooldown_blocks_execution() {
        let decision = ExecutionDecision::determine(
            AutonomyTier::Autonomous,
            ConfidenceTier::High,
            true,  // in cooldown
        );
        assert_eq!(decision, ExecutionDecision::Cooldown);
    }

    #[test]
    fn test_promotion_criteria() {
        let mgr = AutonomyManager {
            tier: AutonomyTier::Observer,
            confidence_map: HashMap::new(),
            config_path: PathBuf::from("/tmp/test_autonomy.json"),
            confidence_path: PathBuf::from("/tmp/test_confidence.json"),
        };

        // Good conditions
        assert!(mgr.check_promotion_criteria(9, false, 5.0));

        // Poor health
        assert!(!mgr.check_promotion_criteria(6, false, 5.0));

        // Critical anomalies
        assert!(!mgr.check_promotion_criteria(9, true, 5.0));

        // High performance drift
        assert!(!mgr.check_promotion_criteria(9, false, 15.0));
    }
}
