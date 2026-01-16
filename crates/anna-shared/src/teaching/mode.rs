//! Teaching Mode v1 - Service desk teaching mirror.
//!
//! v0.3.73: Teaching Mode v1 implementation.
//!
//! # Purpose
//!
//! Teach users how a real service desk reasons, without executing commands,
//! without fixing things for them, and without risking the system.
//!
//! # Hard Constraints
//!
//! - No new execution capabilities
//! - No shell commands
//! - No fixes performed implicitly
//! - No proactive teaching
//! - No gamification output
//! - No "you should do X"
//! - No invented causes or solutions
//! - Teaching is explanation only, grounded in observed system state
//!
//! # When Teaching Mode Activates
//!
//! Teaching Mode activates only when:
//! 1. Explicitly requested by user
//! 2. Question is classified as HowQuestion or WhyQuestion
//! 3. GeneralLinuxQuestion that ties to current system state
//!
//! # Input Classification
//!
//! ```text
//! StatusQuestion     -> handled as today (data retrieval)
//! ChangeQuestion     -> handled by Interpretation Mode
//! HowQuestion        -> Teaching Mode
//! WhyQuestion        -> Teaching Mode
//! FixRequest         -> existing ActionRequest flow
//! GeneralLinuxQuestion -> Teaching Mode ONLY if tied to current system state
//! ```
//!
//! # Teaching Mode Responsibilities
//!
//! 1. **Explain like a service desk**
//!    - What signals would be checked
//!    - Why those signals matter
//!    - What conclusions are supported by evidence
//!    - Explicitly state what is unknown
//!
//! 2. **Ground explanations in reality**
//!    - Always reference: current system state, known baselines, observed diffs
//!    - If no evidence exists, say so clearly
//!
//! 3. **Teach patterns, not steps**
//!    - Explain why something matters before how
//!    - Never provide commands
//!    - Never provide step-by-step fixes
//!    - Never suggest actions
//!
//! 4. **Silent learning hook**
//!    - When user later resolves issue themselves
//!    - Interpretation Mode observes it
//!    - Teaching Mode may later reference that pattern
//!    - No XP, badges, or scores
//!
//! # Explicit Non-Goals
//!
//! - Do not replace documentation
//! - Do not act as a tutor
//! - Do not optimize for speed
//! - Do not optimize for friendliness
//!
//! # Output Rules
//!
//! - Calm
//! - Factual
//! - Boring in the best way
//! - Ends when explanation is complete

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Teaching Mode question types (refined classification).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TeachingQuestion {
    /// Status question - retrieve current state
    /// Routes to: existing data retrieval
    #[default]
    Status,

    /// Change question - what changed, when, why
    /// Routes to: Interpretation Mode
    Change,

    /// How question - how does X work, how would you diagnose Y
    /// Routes to: Teaching Mode
    How,

    /// Why question - why is X happening, why does Y matter
    /// Routes to: Teaching Mode
    Why,

    /// Fix request - install, enable, configure, fix
    /// Routes to: existing ActionRequest flow
    FixRequest,

    /// General Linux question - only teach if tied to system state
    /// Routes to: Teaching Mode (with grounding check)
    GeneralLinux,
}

impl TeachingQuestion {
    /// Whether this question type routes to Teaching Mode.
    pub fn routes_to_teaching(&self) -> bool {
        matches!(self, Self::How | Self::Why | Self::GeneralLinux)
    }

    /// Whether this question requires grounding in system state.
    pub fn requires_grounding(&self) -> bool {
        matches!(self, Self::How | Self::Why | Self::GeneralLinux)
    }

    /// Whether this question routes to Interpretation Mode.
    pub fn routes_to_interpretation(&self) -> bool {
        matches!(self, Self::Change)
    }

    /// Whether this question routes to action flow.
    pub fn routes_to_action(&self) -> bool {
        matches!(self, Self::FixRequest)
    }
}

/// Grounding context for teaching explanations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroundingContext {
    /// Current system state evidence
    pub system_state: Vec<StateEvidence>,
    /// Known baselines referenced
    pub baselines: Vec<String>,
    /// Observed diffs referenced
    pub diffs: Vec<String>,
    /// What we don't know (explicit)
    pub unknowns: Vec<String>,
}

/// A piece of system state evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEvidence {
    /// What was observed
    pub observation: String,
    /// Source of the observation
    pub source: EvidenceSource,
    /// When it was observed
    pub observed_at: DateTime<Utc>,
}

/// Source of state evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceSource {
    /// From SystemBaseline snapshot
    Baseline,
    /// From IssueStore warning
    IssueStore,
    /// From outcome ledger
    OutcomeLedger,
    /// From probe output
    Probe { command: String },
    /// From file content
    FileContent { path: String },
}

impl EvidenceSource {
    /// Format for display.
    pub fn display(&self) -> String {
        match self {
            Self::Baseline => "[baseline snapshot]".to_string(),
            Self::IssueStore => "[issue store]".to_string(),
            Self::OutcomeLedger => "[outcome ledger]".to_string(),
            Self::Probe { command } => format!("[probe: {}]", command),
            Self::FileContent { path } => format!("[file: {}]", path),
        }
    }
}

/// Teaching Mode output structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingOutput {
    /// The teaching explanation
    pub explanation: TeachingExplanation,
    /// Grounding context used
    pub grounding: GroundingContext,
    /// Whether this was fully grounded or has gaps
    pub fully_grounded: bool,
}

/// The actual teaching explanation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingExplanation {
    /// What signals a service desk would check
    pub signals_to_check: Vec<String>,
    /// Why those signals matter
    pub why_signals_matter: String,
    /// Conclusions supported by evidence
    pub conclusions: Vec<EvidencedConclusion>,
    /// What is unknown (explicit)
    pub unknowns: Vec<String>,
}

/// A conclusion supported by evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencedConclusion {
    /// The conclusion
    pub conclusion: String,
    /// The evidence supporting it
    pub evidence: String,
    /// Confidence level
    pub confidence: ConclusionConfidence,
}

/// Confidence in a conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConclusionConfidence {
    /// Strong evidence supports this
    Supported,
    /// Some evidence, not conclusive
    Partial,
    /// No direct evidence, inferred
    Inferred,
}

/// Format a teaching output for display.
pub fn format_teaching_output(output: &TeachingOutput) -> String {
    let mut result = String::new();

    result.push_str("SERVICE DESK PERSPECTIVE\n");
    result.push_str("========================\n\n");

    // Signals to check
    if !output.explanation.signals_to_check.is_empty() {
        result.push_str("Signals a service desk would examine:\n");
        for signal in &output.explanation.signals_to_check {
            result.push_str(&format!("  - {}\n", signal));
        }
        result.push('\n');
    }

    // Why these signals matter
    if !output.explanation.why_signals_matter.is_empty() {
        result.push_str("Why these signals matter:\n");
        result.push_str(&format!("  {}\n\n", output.explanation.why_signals_matter));
    }

    // Conclusions with evidence
    if !output.explanation.conclusions.is_empty() {
        result.push_str("Conclusions from available evidence:\n");
        for conclusion in &output.explanation.conclusions {
            let confidence = match conclusion.confidence {
                ConclusionConfidence::Supported => "supported",
                ConclusionConfidence::Partial => "partial evidence",
                ConclusionConfidence::Inferred => "inferred",
            };
            result.push_str(&format!(
                "  - {} ({})\n    Evidence: {}\n",
                conclusion.conclusion, confidence, conclusion.evidence
            ));
        }
        result.push('\n');
    }

    // Explicit unknowns
    if !output.explanation.unknowns.is_empty() {
        result.push_str("What is unknown:\n");
        for unknown in &output.explanation.unknowns {
            result.push_str(&format!("  - {}\n", unknown));
        }
        result.push('\n');
    }

    // Grounding summary
    if !output.grounding.system_state.is_empty() {
        result.push_str("Evidence sources used:\n");
        for evidence in &output.grounding.system_state {
            result.push_str(&format!("  - {} {}\n", evidence.source.display(), evidence.observation));
        }
        result.push('\n');
    }

    // Status
    if !output.fully_grounded {
        result.push_str("[Partial grounding: some evidence unavailable]\n");
    }

    result.push_str("[End of teaching output]\n");

    result
}

//------------------------------------------------------------------------------
// TEACHING MODE INTERNAL SPEC
//------------------------------------------------------------------------------
//
// ## State Machine
//
// ```text
// [Question Received] --> [Classify Question Type]
//                               |
//       +-----------------------+-----------------------+
//       |           |           |           |           |
//       v           v           v           v           v
//   [Status]   [Change]     [How]      [Why]     [FixRequest]
//       |           |           |           |           |
//       v           v           v           v           v
//   [Data     [Interp.    [Teaching  [Teaching  [Action
//    Retrieval] Mode]       Mode]      Mode]      Flow]
//                               |           |
//                               +-----+-----+
//                                     |
//                                     v
//                            [Gather Grounding]
//                                     |
//                                     v
//                            [Generate Explanation]
//                                     |
//                                     v
//                            [Format Output]
// ```
//
// ## Grounding Requirements
//
// Before generating any teaching output, Teaching Mode must:
// 1. Query SystemBaseline for current state
// 2. Query IssueStore for active warnings
// 3. Query outcome ledger for recent actions
// 4. Identify what evidence exists vs what is missing
//
// ## Output Generation Rules
//
// 1. Start with "why" (why signals matter)
// 2. List signals a service desk would check
// 3. State conclusions with evidence citations
// 4. Explicitly state unknowns
// 5. Never suggest actions
// 6. Never provide commands
// 7. End cleanly
//
// ## Forbidden Outputs
//
// - "You should..."
// - "Try running..."
// - "I recommend..."
// - "The fix is..."
// - Any shell commands
// - Step-by-step instructions
// - Proactive suggestions
//
//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_routing() {
        assert!(!TeachingQuestion::Status.routes_to_teaching());
        assert!(!TeachingQuestion::Change.routes_to_teaching());
        assert!(TeachingQuestion::How.routes_to_teaching());
        assert!(TeachingQuestion::Why.routes_to_teaching());
        assert!(TeachingQuestion::GeneralLinux.routes_to_teaching());
        assert!(!TeachingQuestion::FixRequest.routes_to_teaching());
    }

    #[test]
    fn test_interpretation_routing() {
        assert!(TeachingQuestion::Change.routes_to_interpretation());
        assert!(!TeachingQuestion::How.routes_to_interpretation());
    }

    #[test]
    fn test_action_routing() {
        assert!(TeachingQuestion::FixRequest.routes_to_action());
        assert!(!TeachingQuestion::How.routes_to_action());
    }

    #[test]
    fn test_grounding_requirements() {
        assert!(TeachingQuestion::How.requires_grounding());
        assert!(TeachingQuestion::Why.requires_grounding());
        assert!(TeachingQuestion::GeneralLinux.requires_grounding());
        assert!(!TeachingQuestion::Status.requires_grounding());
    }

    #[test]
    fn test_evidence_source_display() {
        assert_eq!(EvidenceSource::Baseline.display(), "[baseline snapshot]");
        assert_eq!(EvidenceSource::IssueStore.display(), "[issue store]");
        assert_eq!(
            EvidenceSource::Probe { command: "df -h".to_string() }.display(),
            "[probe: df -h]"
        );
    }

    #[test]
    fn test_teaching_output_format() {
        let output = TeachingOutput {
            explanation: TeachingExplanation {
                signals_to_check: vec!["File modification time".to_string()],
                why_signals_matter: "Shows when the file was last changed".to_string(),
                conclusions: vec![EvidencedConclusion {
                    conclusion: "File was modified after package update".to_string(),
                    evidence: "mtime is newer than package manager log entry".to_string(),
                    confidence: ConclusionConfidence::Supported,
                }],
                unknowns: vec!["Which process made the change".to_string()],
            },
            grounding: GroundingContext {
                system_state: vec![StateEvidence {
                    observation: "/etc/group modified".to_string(),
                    source: EvidenceSource::Baseline,
                    observed_at: Utc::now(),
                }],
                baselines: vec!["group file hash".to_string()],
                diffs: vec![],
                unknowns: vec![],
            },
            fully_grounded: true,
        };

        let formatted = format_teaching_output(&output);

        // Check structure
        assert!(formatted.contains("SERVICE DESK PERSPECTIVE"));
        assert!(formatted.contains("Signals a service desk would examine"));
        assert!(formatted.contains("Why these signals matter"));
        assert!(formatted.contains("Conclusions from available evidence"));
        assert!(formatted.contains("What is unknown"));
        assert!(formatted.contains("[End of teaching output]"));

        // Check forbidden content is NOT present
        assert!(!formatted.contains("You should"));
        assert!(!formatted.contains("Try running"));
        assert!(!formatted.contains("sudo"));
        assert!(!formatted.contains("pacman"));
    }
}
