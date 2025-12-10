//! Service desk result types (v0.0.298).
//! v0.0.298: Added `validated` field for proper verification tracking.

use serde::{Deserialize, Serialize};

use super::routing::{SpecialistDomain, TranslatorTicket};
use crate::clarify_v2::ClarifyRequest;
use crate::recipe_feedback::FeedbackRequest;
use crate::reliability::ReliabilityExplanation;
use crate::trace::ExecutionTrace;
use crate::transcript::Transcript;

/// Structured probe result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    /// Command that was run
    pub command: String,
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// First N lines of stdout
    pub stdout: String,
    /// First N lines of stderr
    pub stderr: String,
    /// Execution time in milliseconds
    pub timing_ms: u64,
}

/// Evidence block showing what data was used
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceBlock {
    /// Hardware snapshot fields used
    #[serde(default)]
    pub hardware_fields: Vec<String>,
    /// Probes that were executed
    #[serde(default)]
    pub probes_executed: Vec<ProbeResult>,
    /// Translator ticket that routed this query
    #[serde(default)]
    pub translator_ticket: TranslatorTicket,
    /// Last error if any (e.g., "timeout at translator")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_error: Option<String>,
}

/// Reliability scoring signals (all boolean for deterministic calculation)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReliabilitySignals {
    /// Translator confidence >= 0.7
    #[serde(default)]
    pub translator_confident: bool,
    /// All requested probes succeeded
    #[serde(default)]
    pub probe_coverage: bool,
    /// Answer references probe/hardware data
    #[serde(default)]
    pub answer_grounded: bool,
    /// No invented facts detected
    #[serde(default)]
    pub no_invention: bool,
    /// No clarification needed
    #[serde(default)]
    pub clarification_not_needed: bool,
}

impl ReliabilitySignals {
    /// Calculate score: 20 points per signal, max 100
    pub fn score(&self) -> u8 {
        let mut score: u8 = 0;
        if self.translator_confident {
            score += 20;
        }
        if self.probe_coverage {
            score += 20;
        }
        if self.answer_grounded {
            score += 20;
        }
        if self.no_invention {
            score += 20;
        }
        if self.clarification_not_needed {
            score += 20;
        }
        score
    }
}

/// Unified response from service desk pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDeskResult {
    /// Unique request ID for tracking
    pub request_id: String,
    /// v0.0.106: Case number (e.g., "CN-0001-06122025")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    /// v0.0.106: Staff member who handled this request (display string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_staff: Option<String>,
    /// v0.0.109: Staff person ID for lookup (e.g., "desktop_jr")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff_id: Option<String>,
    /// The LLM's answer text
    pub answer: String,
    /// v0.0.298: Whether the answer passed validation (proper ticket loop verification)
    /// This is more authoritative than reliability_score >= threshold.
    #[serde(default)]
    pub validated: bool,
    /// Reliability score 0-100 (deterministic from signals)
    pub reliability_score: u8,
    /// Reliability scoring signals
    pub reliability_signals: ReliabilitySignals,
    /// TRUST: Structured explanation when score < 80 (None otherwise)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reliability_explanation: Option<ReliabilityExplanation>,
    /// Which specialist handled this
    pub domain: SpecialistDomain,
    /// Evidence block showing data sources
    pub evidence: EvidenceBlock,
    /// Whether clarification is needed
    pub needs_clarification: bool,
    /// Question to ask if clarification needed (legacy)
    pub clarification_question: Option<String>,
    /// Full clarification request with options (v0.0.47+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clarification_request: Option<ClarifyRequest>,
    /// Full transcript of pipeline events
    pub transcript: Transcript,
    /// TRACE: Execution trace showing stages and paths (v0.0.23+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_trace: Option<ExecutionTrace>,
    /// v0.0.96: Proposed config change requiring user confirmation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposed_change: Option<crate::change::ChangePlan>,
    /// v0.0.136+: Proposed config changes (supports multi-line/multi-step)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposed_changes: Vec<crate::change::ChangePlan>,
    /// v0.0.103: Anna asks for feedback when uncertain about recipe answer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_request: Option<FeedbackRequest>,
}
