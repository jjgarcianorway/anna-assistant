//! Conscience subsystem type definitions
//!
//! Phase 1.1: Self-reflective ethical governance
//! Citation: [archwiki:System_maintenance]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::sentinel::SentinelAction;

/// Conscience state persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConscienceState {
    /// State version for migration tracking
    pub version: u64,
    /// State snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// Actions pending manual review
    pub pending_actions: Vec<PendingAction>,
    /// History of all conscience decisions
    pub decision_history: Vec<ConscienceDecision>,
    /// Detected ethical violations
    pub ethical_violations: Vec<EthicalViolation>,
    /// Introspection metrics
    pub introspection_runs: u64,
    pub last_introspection: Option<DateTime<Utc>>,
}

impl Default for ConscienceState {
    fn default() -> Self {
        Self {
            version: 1,
            timestamp: Utc::now(),
            pending_actions: Vec::new(),
            decision_history: Vec::new(),
            ethical_violations: Vec::new(),
            introspection_runs: 0,
            last_introspection: None,
        }
    }
}

/// Action pending manual review due to uncertainty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    /// Unique identifier
    pub id: String,
    /// When action was proposed
    pub timestamp: DateTime<Utc>,
    /// The action to be taken
    pub action: SentinelAction,
    /// Why it was flagged
    pub flag_reason: String,
    /// Uncertainty score (0.0-1.0)
    pub uncertainty: f64,
    /// Ethical evaluation
    pub ethical_score: EthicalScore,
    /// Reasoning chain
    pub reasoning: ReasoningTree,
}

/// A conscience decision with full audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConscienceDecision {
    /// Unique identifier
    pub id: String,
    /// Decision timestamp
    pub timestamp: DateTime<Utc>,
    /// The action evaluated
    pub action: SentinelAction,
    /// Reasoning tree explaining decision
    pub reasoning: ReasoningTree,
    /// Ethical evaluation scores
    pub ethical_score: EthicalScore,
    /// Confidence in decision (0.0-1.0)
    pub confidence: f64,
    /// Decision outcome
    pub outcome: DecisionOutcome,
    /// Rollback plan if applicable
    pub rollback_plan: Option<RollbackPlan>,
}

/// Ethical evaluation across four dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalScore {
    /// Safety: Will this harm the system or data?
    pub safety: f64,
    /// Privacy: Does this respect user data boundaries?
    pub privacy: f64,
    /// Integrity: Is this action honest and transparent?
    pub integrity: f64,
    /// Autonomy: Does this preserve user control?
    pub autonomy: f64,
}

impl EthicalScore {
    /// Calculate overall ethical score (0.0-1.0)
    pub fn overall(&self) -> f64 {
        (self.safety + self.privacy + self.integrity + self.autonomy) / 4.0
    }

    /// Check if score passes ethical threshold
    pub fn is_ethical(&self, threshold: f64) -> bool {
        self.overall() >= threshold && self.safety >= threshold
    }

    /// Get lowest dimension score
    pub fn min_score(&self) -> f64 {
        self.safety
            .min(self.privacy)
            .min(self.integrity)
            .min(self.autonomy)
    }

    /// Get dimension name with lowest score
    pub fn weakest_dimension(&self) -> &'static str {
        let min = self.min_score();
        if self.safety == min {
            "safety"
        } else if self.privacy == min {
            "privacy"
        } else if self.integrity == min {
            "integrity"
        } else {
            "autonomy"
        }
    }
}

/// Hierarchical reasoning tree for explainability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTree {
    /// Root reasoning node
    pub root: ReasoningNode,
}

impl ReasoningTree {
    /// Create new reasoning tree with root statement
    pub fn new(statement: String) -> Self {
        Self {
            root: ReasoningNode {
                statement,
                evidence: Vec::new(),
                confidence: 1.0,
                children: Vec::new(),
            },
        }
    }

    /// Calculate overall confidence from tree
    pub fn overall_confidence(&self) -> f64 {
        self.root.subtree_confidence()
    }
}

/// Node in reasoning tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningNode {
    /// Statement or claim
    pub statement: String,
    /// Evidence supporting statement
    pub evidence: Vec<String>,
    /// Confidence in this node (0.0-1.0)
    pub confidence: f64,
    /// Child reasoning nodes
    pub children: Vec<ReasoningNode>,
}

impl ReasoningNode {
    /// Calculate confidence including all children
    pub fn subtree_confidence(&self) -> f64 {
        if self.children.is_empty() {
            self.confidence
        } else {
            let child_avg: f64 = self
                .children
                .iter()
                .map(|c| c.subtree_confidence())
                .sum::<f64>()
                / self.children.len() as f64;
            (self.confidence + child_avg) / 2.0
        }
    }
}

/// Outcome of conscience decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionOutcome {
    /// Action approved and executed
    Approved { execution_result: String },
    /// Action rejected by conscience
    Rejected { reason: String },
    /// Action flagged for manual review
    Flagged { reason: String },
    /// Action pending (not yet decided)
    Pending,
}

/// Plan for rolling back a potentially destructive action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    /// Unique plan identifier
    pub id: String,
    /// Description of rollback procedure
    pub description: String,
    /// Commands to execute for rollback
    pub rollback_commands: Vec<String>,
    /// Pre-action checksums for verification
    pub checksums: HashMap<String, String>,
    /// Backup paths created
    pub backup_paths: Vec<String>,
    /// Estimated rollback time in seconds
    pub estimated_time: u64,
}

/// Detected ethical violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalViolation {
    /// When violation was detected
    pub timestamp: DateTime<Utc>,
    /// Action that caused violation
    pub action: SentinelAction,
    /// Dimension that was violated
    pub dimension: String,
    /// Score that failed threshold
    pub score: f64,
    /// Threshold that was expected
    pub threshold: f64,
    /// Description of violation
    pub description: String,
}

/// Configuration for ethical evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsConfig {
    /// Minimum ethical score required (0.0-1.0)
    pub ethical_threshold: f64,
    /// Maximum uncertainty allowed (0.0-1.0)
    pub uncertainty_threshold: f64,
    /// Minimum confidence required (0.0-1.0)
    pub confidence_threshold: f64,
    /// Require rollback plan for destructive actions
    pub require_rollback: bool,
    /// Enabled ethical dimensions
    pub enabled_dimensions: Vec<String>,
    /// Custom rules
    pub custom_rules: HashMap<String, String>,
}

impl Default for EthicsConfig {
    fn default() -> Self {
        Self {
            ethical_threshold: 0.7,
            uncertainty_threshold: 0.4,
            confidence_threshold: 0.6,
            require_rollback: true,
            enabled_dimensions: vec![
                "safety".to_string(),
                "privacy".to_string(),
                "integrity".to_string(),
                "autonomy".to_string(),
            ],
            custom_rules: HashMap::new(),
        }
    }
}

/// Journal entry for append-only audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Entry timestamp
    pub timestamp: DateTime<Utc>,
    /// Decision ID
    pub decision_id: String,
    /// Action type
    pub action_type: String,
    /// Decision outcome
    pub outcome: String,
    /// Ethical score
    pub ethical_score: f64,
    /// Confidence
    pub confidence: f64,
    /// Summary
    pub summary: String,
}

/// Introspection report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionReport {
    /// Report timestamp
    pub timestamp: DateTime<Utc>,
    /// Time period analyzed
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    /// Total decisions reviewed
    pub decisions_reviewed: u64,
    /// Approved actions
    pub approved_count: u64,
    /// Rejected actions
    pub rejected_count: u64,
    /// Flagged actions
    pub flagged_count: u64,
    /// Average ethical score
    pub avg_ethical_score: f64,
    /// Average confidence
    pub avg_confidence: f64,
    /// Ethical violations detected
    pub violations: Vec<EthicalViolation>,
    /// Recommendations
    pub recommendations: Vec<String>,
}
