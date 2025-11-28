//! Idle Learning Engine for v0.23.0
//!
//! Implements background learning that runs during system idle time:
//! - Learning missions queued by Junior/Senior
//! - Safe execution within resource limits
//! - Fact discovery from probes and file scans

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

use super::protocol_v23::{KnowledgeScope, FactSourceV23};

// ============================================================================
// RESOURCE LIMITS
// ============================================================================

/// Default maximum probes per idle cycle
pub const DEFAULT_MAX_PROBES_PER_CYCLE: usize = 3;

/// Default maximum files scanned per cycle
pub const DEFAULT_MAX_FILES_PER_CYCLE: usize = 10;

/// Default maximum bytes read per cycle
pub const DEFAULT_MAX_BYTES_PER_CYCLE: usize = 1024 * 1024; // 1 MB

/// Default maximum scan time per cycle (milliseconds)
pub const DEFAULT_MAX_SCAN_TIME_MS: u64 = 5000; // 5 seconds

/// Default CPU load threshold (percentage)
pub const DEFAULT_CPU_THRESHOLD: f32 = 30.0;

/// Default idle check interval (seconds)
pub const DEFAULT_IDLE_INTERVAL_SECS: u64 = 60;

// ============================================================================
// IDLE LEARNING CONFIG
// ============================================================================

/// Configuration for the idle learning engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleLearningConfig {
    /// Whether idle learning is enabled
    pub enabled: bool,
    /// Maximum probes per idle cycle
    pub max_probes_per_cycle: usize,
    /// Maximum files scanned per cycle
    pub max_files_per_cycle: usize,
    /// Maximum bytes read per cycle
    pub max_bytes_per_cycle: usize,
    /// Maximum scan time per cycle (milliseconds)
    pub max_scan_time_ms: u64,
    /// CPU load threshold (percentage) - only run when below this
    pub cpu_threshold: f32,
    /// Interval between idle checks (seconds)
    pub idle_interval_secs: u64,
}

impl Default for IdleLearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_probes_per_cycle: DEFAULT_MAX_PROBES_PER_CYCLE,
            max_files_per_cycle: DEFAULT_MAX_FILES_PER_CYCLE,
            max_bytes_per_cycle: DEFAULT_MAX_BYTES_PER_CYCLE,
            max_scan_time_ms: DEFAULT_MAX_SCAN_TIME_MS,
            cpu_threshold: DEFAULT_CPU_THRESHOLD,
            idle_interval_secs: DEFAULT_IDLE_INTERVAL_SECS,
        }
    }
}

// ============================================================================
// LEARNING MISSION
// ============================================================================

/// A learning mission - a small, concrete task to discover facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMission {
    /// Unique mission identifier
    pub id: String,
    /// Target scope (system or user)
    pub target_scope: KnowledgeScope,
    /// Human-readable description
    pub description: String,
    /// Fact keys this mission wants to fill
    pub required_fact_keys: Vec<String>,
    /// Probes to use (from catalog)
    pub probes_to_use: Vec<String>,
    /// File patterns to scan (optional)
    pub file_patterns: Vec<String>,
    /// Priority (lower = higher priority)
    pub priority: u8,
    /// Number of times this mission has failed
    pub failure_count: u32,
    /// Maximum failures before dropping
    pub max_failures: u32,
    /// When this mission was created (Unix timestamp)
    pub created_at: i64,
    /// When this mission was last attempted (Unix timestamp)
    pub last_attempted_at: Option<i64>,
}

impl LearningMission {
    /// Create a new learning mission
    pub fn new(
        id: String,
        target_scope: KnowledgeScope,
        description: String,
        required_fact_keys: Vec<String>,
    ) -> Self {
        Self {
            id,
            target_scope,
            description,
            required_fact_keys,
            probes_to_use: Vec::new(),
            file_patterns: Vec::new(),
            priority: 50, // Default middle priority
            failure_count: 0,
            max_failures: 3,
            created_at: chrono::Utc::now().timestamp(),
            last_attempted_at: None,
        }
    }

    /// Add probes to this mission
    pub fn with_probes(mut self, probes: Vec<String>) -> Self {
        self.probes_to_use = probes;
        self
    }

    /// Add file patterns to this mission
    pub fn with_file_patterns(mut self, patterns: Vec<String>) -> Self {
        self.file_patterns = patterns;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Check if mission should be dropped due to failures
    pub fn should_drop(&self) -> bool {
        self.failure_count >= self.max_failures
    }

    /// Record a failure
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_attempted_at = Some(chrono::Utc::now().timestamp());
    }

    /// Record an attempt
    pub fn record_attempt(&mut self) {
        self.last_attempted_at = Some(chrono::Utc::now().timestamp());
    }
}

/// Status of a mission execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    /// Mission completed successfully
    Success,
    /// Mission failed but can retry
    Failed { reason: String },
    /// Mission skipped (e.g., facts already exist)
    Skipped { reason: String },
    /// Mission dropped (too many failures)
    Dropped,
    /// Mission pending execution
    Pending,
}

// ============================================================================
// MISSION QUEUE
// ============================================================================

/// Queue of learning missions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MissionQueue {
    /// Pending missions (priority queue)
    pub pending: VecDeque<LearningMission>,
    /// Completed mission IDs (for deduplication)
    pub completed_ids: Vec<String>,
    /// Maximum completed IDs to keep
    pub max_completed_history: usize,
}

impl MissionQueue {
    /// Create a new mission queue
    pub fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            completed_ids: Vec::new(),
            max_completed_history: 100,
        }
    }

    /// Add a mission to the queue
    pub fn enqueue(&mut self, mission: LearningMission) {
        // Skip if already completed
        if self.completed_ids.contains(&mission.id) {
            return;
        }

        // Skip if already pending
        if self.pending.iter().any(|m| m.id == mission.id) {
            return;
        }

        // Insert by priority (lower = higher priority)
        let pos = self.pending.iter()
            .position(|m| m.priority > mission.priority)
            .unwrap_or(self.pending.len());
        self.pending.insert(pos, mission);
    }

    /// Pop the next mission
    pub fn pop(&mut self) -> Option<LearningMission> {
        self.pending.pop_front()
    }

    /// Mark a mission as completed
    pub fn mark_completed(&mut self, mission_id: &str) {
        self.completed_ids.push(mission_id.to_string());

        // Trim history if needed
        while self.completed_ids.len() > self.max_completed_history {
            self.completed_ids.remove(0);
        }
    }

    /// Re-queue a failed mission
    pub fn requeue_failed(&mut self, mut mission: LearningMission) {
        mission.record_failure();

        if mission.should_drop() {
            // Don't requeue - mark as completed to avoid retrying
            self.mark_completed(&mission.id);
        } else {
            // Increase priority (lower number) slightly for retry
            mission.priority = mission.priority.saturating_sub(5);
            self.pending.push_back(mission);
        }
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

// ============================================================================
// IDLE STATE
// ============================================================================

/// State of the idle learning engine
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdleState {
    /// Engine is idle, waiting for conditions
    Waiting,
    /// Checking if conditions are met
    Checking,
    /// Executing a mission
    Executing { mission_id: String },
    /// Paused (e.g., user request in progress)
    Paused,
    /// Disabled
    Disabled,
}

/// Conditions for idle learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleConditions {
    /// Current CPU load percentage
    pub cpu_load: f32,
    /// Whether a user request is active
    pub request_active: bool,
    /// Seconds since last user activity
    pub idle_seconds: u64,
    /// Whether conditions are met for learning
    pub conditions_met: bool,
}

impl IdleConditions {
    /// Check if conditions are met
    pub fn check(cpu_load: f32, request_active: bool, config: &IdleLearningConfig) -> Self {
        let conditions_met = !request_active
            && cpu_load < config.cpu_threshold
            && config.enabled;

        Self {
            cpu_load,
            request_active,
            idle_seconds: 0, // To be filled by caller
            conditions_met,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_config_default() {
        let config = IdleLearningConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_probes_per_cycle, 3);
        assert_eq!(config.cpu_threshold, 30.0);
    }

    #[test]
    fn test_mission_creation() {
        let mission = LearningMission::new(
            "test-001".to_string(),
            KnowledgeScope::User,
            "Find nvim config".to_string(),
            vec!["user.editor.nvim.config_main".to_string()],
        ).with_probes(vec!["pacman_qi".to_string()])
         .with_file_patterns(vec!["~/.config/nvim/**/init.*".to_string()]);

        assert_eq!(mission.id, "test-001");
        assert_eq!(mission.target_scope, KnowledgeScope::User);
        assert!(!mission.probes_to_use.is_empty());
        assert!(!mission.file_patterns.is_empty());
    }

    #[test]
    fn test_mission_queue() {
        let mut queue = MissionQueue::new();

        let m1 = LearningMission::new(
            "m1".to_string(),
            KnowledgeScope::System,
            "Mission 1".to_string(),
            vec![],
        ).with_priority(50);

        let m2 = LearningMission::new(
            "m2".to_string(),
            KnowledgeScope::System,
            "Mission 2".to_string(),
            vec![],
        ).with_priority(10); // Higher priority

        queue.enqueue(m1);
        queue.enqueue(m2);

        // m2 should come first (lower priority number)
        let popped = queue.pop().unwrap();
        assert_eq!(popped.id, "m2");
    }

    #[test]
    fn test_mission_failure_tracking() {
        let mut mission = LearningMission::new(
            "test".to_string(),
            KnowledgeScope::User,
            "Test".to_string(),
            vec![],
        );
        mission.max_failures = 2;

        assert!(!mission.should_drop());
        mission.record_failure();
        assert!(!mission.should_drop());
        mission.record_failure();
        assert!(mission.should_drop());
    }

    #[test]
    fn test_idle_conditions() {
        let config = IdleLearningConfig::default();

        // Low CPU, no request = good
        let cond = IdleConditions::check(10.0, false, &config);
        assert!(cond.conditions_met);

        // High CPU = bad
        let cond = IdleConditions::check(50.0, false, &config);
        assert!(!cond.conditions_met);

        // Request active = bad
        let cond = IdleConditions::check(10.0, true, &config);
        assert!(!cond.conditions_met);
    }
}
