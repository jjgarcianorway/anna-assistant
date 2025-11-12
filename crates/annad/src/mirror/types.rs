//! Mirror Protocol types - Recursive introspection and self-validation
//!
//! Phase 1.4: Metacognition and ethical coherence validation
//! Citation: [archwiki:System_maintenance]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Peer ID type
pub type PeerId = String;

/// Reflection report - compact record of ethical/empathic decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionReport {
    /// Report ID
    pub id: String,
    /// Node that generated this reflection
    pub node_id: PeerId,
    /// Timestamp of reflection
    pub timestamp: DateTime<Utc>,
    /// Time period covered by this reflection
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    /// Ethical decisions made during period
    pub ethical_decisions: Vec<EthicalDecisionRecord>,
    /// Empathy state summary
    pub empathy_summary: EmpathySummary,
    /// Conscience actions taken
    pub conscience_actions: Vec<ConscienceActionRecord>,
    /// Trust score deltas (changes in peer trust)
    pub trust_deltas: HashMap<PeerId, f64>,

    /// Self-assessed ethical coherence (0.0-1.0)
    pub self_coherence: f64,
    /// Detected biases or concerns
    pub self_identified_biases: Vec<String>,
    /// Signature (for authenticity)
    pub signature: String,
}

/// Record of an ethical decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalDecisionRecord {
    /// Decision ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Action being evaluated
    pub action: String,
    /// Decision outcome (Approved, Rejected, Deferred)
    pub outcome: String,
    /// Ethical score
    pub ethical_score: f64,
    /// Reasoning
    pub reasoning: String,
    /// Stakeholder impacts
    pub stakeholder_impacts: HashMap<String, f64>,
}

/// Summary of empathy state over reflection period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpathySummary {
    /// Average empathy index
    pub avg_empathy_index: f64,
    /// Average strain index
    pub avg_strain_index: f64,
    /// Empathy trend (increasing, decreasing, stable)
    pub empathy_trend: String,
    /// Strain trend
    pub strain_trend: String,
    /// Number of adaptations made
    pub adaptations_count: usize,
}

/// Record of conscience action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConscienceActionRecord {
    /// Action ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Action type
    pub action_type: String,
    /// Required human review?
    pub required_review: bool,
    /// Was approved?
    pub approved: bool,
}

/// Peer critique - evaluation of another node's reflection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerCritique {
    /// Critique ID
    pub id: String,
    /// Critic node ID
    pub critic_id: PeerId,
    /// Target reflection ID
    pub reflection_id: String,
    /// Target node ID
    pub target_node_id: PeerId,
    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Ethical coherence assessment (0.0-1.0)
    pub coherence_assessment: f64,
    /// Identified inconsistencies
    pub inconsistencies: Vec<Inconsistency>,
    /// Identified biases
    pub identified_biases: Vec<BiasDetection>,
    /// Overall critique reasoning
    pub reasoning: String,
    /// Recommended corrections
    pub recommendations: Vec<String>,

    /// Critic's trust score at time of critique
    pub critic_trust: f64,
    /// Signature
    pub signature: String,
}

/// Detected inconsistency in behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inconsistency {
    /// Type of inconsistency
    pub inconsistency_type: String,
    /// Description
    pub description: String,
    /// Severity (0.0-1.0)
    pub severity: f64,
    /// Evidence
    pub evidence: Vec<String>,
}

/// Detected bias in decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasDetection {
    /// Bias type (confirmation, recency, authority, etc.)
    pub bias_type: String,
    /// Description
    pub description: String,
    /// Confidence (0.0-1.0)
    pub confidence: f64,
    /// Affected decisions
    pub affected_decisions: Vec<String>,
    /// Recommended correction
    pub correction: String,
}

/// Mirror consensus session - collective alignment evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConsensusSession {
    /// Session ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Participating nodes
    pub participants: Vec<PeerId>,

    /// Reflections analyzed
    pub reflections_analyzed: Vec<String>,
    /// Critiques received
    pub critiques_received: Vec<String>,

    /// Collective ethical trends
    pub ethical_trends: EthicalTrends,
    /// Systemic biases detected
    pub systemic_biases: Vec<SystemicBias>,
    /// Network coherence score (0.0-1.0)
    pub network_coherence: f64,

    /// Consensus outcome
    pub outcome: ConsensusOutcome,
    /// Remediation actions approved
    pub approved_remediations: Vec<RemediationAction>,
}

/// Ethical trends across the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalTrends {
    /// Average ethical coherence
    pub avg_coherence: f64,
    /// Coherence trend (improving, declining, stable)
    pub coherence_trend: String,
    /// Average empathy index across network
    pub avg_empathy: f64,
    /// Average strain index across network
    pub avg_strain: f64,
    /// Common decision patterns
    pub common_patterns: Vec<String>,
}

/// Systemic bias affecting multiple nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemicBias {
    /// Bias type
    pub bias_type: String,
    /// Description
    pub description: String,
    /// Affected nodes
    pub affected_nodes: Vec<PeerId>,
    /// Severity (0.0-1.0)
    pub severity: f64,
    /// Root cause hypothesis
    pub root_cause: String,
}

/// Mirror consensus outcome
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsensusOutcome {
    /// Network is coherent, no action needed
    Coherent,
    /// Minor adjustments recommended
    MinorAdjustment,
    /// Significant remediation required
    SignificantRemediation,
    /// Critical ethical divergence detected
    CriticalDivergence,
}

/// Remediation action for bias correction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationAction {
    /// Action ID
    pub id: String,
    /// Target node (or "all" for network-wide)
    pub target_node: String,
    /// Remediation type
    pub remediation_type: RemediationType,
    /// Description
    pub description: String,
    /// Parameters to adjust
    pub parameter_adjustments: HashMap<String, f64>,
    /// Expected impact
    pub expected_impact: String,
}

/// Type of remediation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemediationType {
    /// Reweight empathy/strain parameters
    ParameterReweight,
    /// Reset trust scores for specific peers
    TrustReset,
    /// Adjust conscience thresholds
    ConscienceAdjustment,
    /// Retrain decision patterns
    PatternRetrain,
    /// Manual review required
    ManualReview,
}

/// Mirror protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConfig {
    /// Is mirror protocol enabled?
    pub enabled: bool,
    /// Reflection interval (hours)
    pub reflection_interval_hours: u64,
    /// Minimum nodes for consensus
    pub min_consensus_nodes: usize,
    /// Coherence threshold for action (0.0-1.0)
    pub coherence_threshold: f64,
    /// Bias confidence threshold (0.0-1.0)
    pub bias_confidence_threshold: f64,
    /// Enable automatic remediation
    pub auto_remediation: bool,
    /// Encryption enabled for reflections
    pub encryption_enabled: bool,
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            reflection_interval_hours: 24,
            min_consensus_nodes: 3,
            coherence_threshold: 0.7,
            bias_confidence_threshold: 0.8,
            auto_remediation: false,
            encryption_enabled: true,
        }
    }
}

/// Mirror protocol state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorState {
    /// Version
    pub version: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Node ID
    pub node_id: PeerId,

    /// Recent reflections
    pub recent_reflections: Vec<ReflectionReport>,
    /// Received critiques
    pub received_critiques: Vec<PeerCritique>,
    /// Consensus sessions
    pub consensus_sessions: Vec<MirrorConsensusSession>,
    /// Applied remediations
    pub applied_remediations: Vec<RemediationAction>,

    /// Current coherence score
    pub current_coherence: f64,
    /// Last reflection timestamp
    pub last_reflection: Option<DateTime<Utc>>,
    /// Last consensus timestamp
    pub last_consensus: Option<DateTime<Utc>>,
}

impl Default for MirrorState {
    fn default() -> Self {
        Self {
            version: 1,
            timestamp: Utc::now(),
            node_id: String::new(),
            recent_reflections: Vec::new(),
            received_critiques: Vec::new(),
            consensus_sessions: Vec::new(),
            applied_remediations: Vec::new(),
            current_coherence: 0.5,
            last_reflection: None,
            last_consensus: None,
        }
    }
}

/// Mirror status for CLI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorStatus {
    /// Is enabled?
    pub enabled: bool,
    /// Current coherence
    pub current_coherence: f64,
    /// Last reflection time
    pub last_reflection: Option<String>,
    /// Last consensus time
    pub last_consensus: Option<String>,
    /// Recent reflections count
    pub recent_reflections_count: usize,
    /// Received critiques count
    pub received_critiques_count: usize,
    /// Active remediations count
    pub active_remediations_count: usize,
    /// Network coherence (if in consensus)
    pub network_coherence: Option<f64>,
}

/// Audit summary for RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub enabled: bool,
    pub current_coherence: f64,
    pub last_reflection: Option<DateTime<Utc>>,
    pub last_consensus: Option<DateTime<Utc>>,
    pub recent_reflections_count: usize,
    pub received_critiques_count: usize,
    pub active_remediations_count: usize,
    pub network_coherence: Option<f64>,
    pub recent_critiques: Vec<SimplifiedCritiqueSummary>,
}

/// Simplified critique summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedCritiqueSummary {
    pub critic_id: PeerId,
    pub coherence_assessment: f64,
    pub inconsistencies_count: usize,
    pub biases_count: usize,
    pub recommendations: Vec<String>,
}
