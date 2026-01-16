//! Teaching Mode - Service desk teaching mirror.
//!
//! v0.3.29: Initial implementation for Milestone 4
//! v0.3.71: Teaching Mode specification - intent classification
//! v0.3.73: Teaching Mode v1 - service desk teaching mirror
//!
//! # Teaching Mode v1
//!
//! Teaches users how a real service desk reasons, without executing commands,
//! without fixing things for them, and without risking the system.
//!
//! ## Hard Constraints
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
//! ## Routing
//!
//! - StatusQuestion -> existing data retrieval
//! - ChangeQuestion -> Interpretation Mode
//! - HowQuestion -> Teaching Mode
//! - WhyQuestion -> Teaching Mode
//! - FixRequest -> existing ActionRequest flow
//! - GeneralLinuxQuestion -> Teaching Mode (only if tied to system state)
//!
//! ## Output Rules
//!
//! - Calm
//! - Factual
//! - Boring in the best way
//! - Ends when explanation is complete

mod tips;
mod intent;
mod mode;
mod grounding;
mod explanation;

pub use tips::{classify_question, format_teaching_block, generate_teaching};
pub use intent::{classify_teaching_intent, classify_teaching_question, TeachingIntent, TeachingResponse, format_servicedesk_reasoning, format_explanation};
pub use mode::{
    TeachingQuestion, TeachingOutput, TeachingExplanation as TeachingExplanationV1,
    GroundingContext, StateEvidence, EvidenceSource, EvidencedConclusion,
    ConclusionConfidence, format_teaching_output,
};
pub use grounding::{gather_grounding, has_sufficient_grounding, report_missing_grounding};
pub use explanation::{generate_teaching_explanation, explain_group_warning};

use serde::{Deserialize, Serialize};

/// A citation source for teaching explanations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CitationSource {
    /// Probe output (command and result summary)
    Probe { command: String, summary: String },
    /// Man page citation
    ManPage { command: String, section: Option<String> },
    /// Arch Wiki citation
    ArchWiki { article: String, section: Option<String> },
    /// Help output citation
    HelpOutput { command: String },
    /// Experiment result
    Experiment { name: String, expected: String, actual: String },
}

impl CitationSource {
    /// Format citation for display (user-facing, no internals)
    pub fn display(&self) -> String {
        match self {
            CitationSource::Probe { command, summary } => {
                format!("[probe: {}] {}", command, summary)
            }
            CitationSource::ManPage { command, section } => {
                if let Some(s) = section {
                    format!("[man {}({})]", command, s)
                } else {
                    format!("[man {}]", command)
                }
            }
            CitationSource::ArchWiki { article, section } => {
                if let Some(s) = section {
                    format!("[Arch Wiki: {} - {}]", article, s)
                } else {
                    format!("[Arch Wiki: {}]", article)
                }
            }
            CitationSource::HelpOutput { command } => {
                format!("[{} --help]", command)
            }
            CitationSource::Experiment { name, expected, actual } => {
                format!("[experiment: {} expected={}, actual={}]", name, expected, actual)
            }
        }
    }
}

/// Context for generating teaching explanations
#[derive(Debug, Clone, Default)]
pub struct TeachingContext {
    /// Question type classification
    pub question_type: QuestionType,
    /// Probes that were executed
    pub probes: Vec<ProbeResult>,
    /// Doc citations available
    pub doc_citations: Vec<CitationSource>,
    /// Experiment results (if any)
    pub experiments: Vec<ExperimentSummary>,
    /// Whether this involved a risky action
    pub is_risky: bool,
    /// Why the action was considered risky (principle, not score)
    pub risk_reason: Option<String>,
}

/// Question type for teaching explanation selection
#[derive(Debug, Clone, Default, PartialEq)]
pub enum QuestionType {
    /// Simple probe query (e.g., "how much RAM?")
    #[default]
    ProbeOnly,
    /// Procedural/semantic question (e.g., "what does systemctl mask do?")
    Procedural,
    /// Action that triggered experiment mode
    RiskyAction,
}

/// Result of a probe execution
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub command: String,
    pub output_summary: String,
    pub success: bool,
}

/// Summary of an experiment
#[derive(Debug, Clone)]
pub struct ExperimentSummary {
    pub name: String,
    pub expected: String,
    pub actual: String,
    pub sandbox_type: String,
}

/// Teaching explanation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingExplanation {
    /// Whether teaching could be generated
    pub success: bool,
    /// The explanation text (1-3 sentences)
    pub explanation: String,
    /// Citations used in the explanation
    pub citations: Vec<String>,
    /// If failed, why
    pub failure_reason: Option<String>,
}

impl TeachingExplanation {
    /// Create a successful teaching explanation
    pub fn success(explanation: String, citations: Vec<String>) -> Self {
        Self {
            success: true,
            explanation,
            citations,
            failure_reason: None,
        }
    }

    /// Create a failed teaching explanation (cannot explain confidently)
    pub fn cannot_explain(reason: &str) -> Self {
        Self {
            success: false,
            explanation: "Cannot explain confidently with local docs available.".to_string(),
            citations: vec![],
            failure_reason: Some(reason.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_only_does_not_require_docs() {
        let ctx = TeachingContext {
            question_type: QuestionType::ProbeOnly,
            probes: vec![ProbeResult {
                command: "free -h".to_string(),
                output_summary: "7.2G available".to_string(),
                success: true,
            }],
            doc_citations: vec![], // No docs - should still work
            ..Default::default()
        };

        let result = generate_teaching(&ctx);
        assert!(result.success);
        assert!(result.explanation.contains("free -h"));
        assert!(!result.citations.is_empty());
    }

    #[test]
    fn test_procedural_requires_doc_citation() {
        // Without docs - should fail
        let ctx_no_docs = TeachingContext {
            question_type: QuestionType::Procedural,
            probes: vec![],
            doc_citations: vec![],
            ..Default::default()
        };

        let result = generate_teaching(&ctx_no_docs);
        assert!(!result.success);
        assert!(result.explanation.contains("Cannot explain confidently"));

        // With docs - should succeed
        let ctx_with_docs = TeachingContext {
            question_type: QuestionType::Procedural,
            probes: vec![],
            doc_citations: vec![CitationSource::ManPage {
                command: "systemctl".to_string(),
                section: Some("1".to_string()),
            }],
            ..Default::default()
        };

        let result = generate_teaching(&ctx_with_docs);
        assert!(result.success);
        assert!(result.citations.iter().any(|c| c.contains("man systemctl")));
    }

    #[test]
    fn test_classify_question_procedural() {
        assert_eq!(classify_question("what does systemctl mask do?"), QuestionType::Procedural);
        assert_eq!(classify_question("explain what systemd is"), QuestionType::Procedural);
        assert_eq!(classify_question("how does pacman work?"), QuestionType::Procedural);
    }

    #[test]
    fn test_classify_question_probe_only() {
        assert_eq!(classify_question("how much free RAM?"), QuestionType::ProbeOnly);
        assert_eq!(classify_question("what kernel am I running?"), QuestionType::ProbeOnly);
        assert_eq!(classify_question("disk usage?"), QuestionType::ProbeOnly);
    }

    #[test]
    fn test_risky_action_needs_citation() {
        // Without citations - should fail
        let ctx_no_cite = TeachingContext {
            question_type: QuestionType::RiskyAction,
            is_risky: true,
            risk_reason: Some("it modifies system services".to_string()),
            experiments: vec![],
            doc_citations: vec![],
            ..Default::default()
        };

        let result = generate_teaching(&ctx_no_cite);
        assert!(!result.success);

        // With experiment - should succeed
        let ctx_with_exp = TeachingContext {
            question_type: QuestionType::RiskyAction,
            is_risky: true,
            risk_reason: Some("it modifies system services".to_string()),
            experiments: vec![ExperimentSummary {
                name: "service-restart".to_string(),
                expected: "success".to_string(),
                actual: "success".to_string(),
                sandbox_type: "dry-run".to_string(),
            }],
            doc_citations: vec![],
            ..Default::default()
        };

        let result = generate_teaching(&ctx_with_exp);
        assert!(result.success);
        assert!(result.citations.iter().any(|c| c.contains("experiment")));
    }

    /// v0.3.29: Mandatory Milestone 4 test
    /// Procedural claims MUST include doc citation
    #[test]
    fn test_teaching_mode_includes_citation_for_procedural_claim() {
        let ctx = TeachingContext {
            question_type: QuestionType::Procedural,
            probes: vec![],
            doc_citations: vec![CitationSource::ManPage {
                command: "systemctl".to_string(),
                section: Some("1".to_string()),
            }],
            ..Default::default()
        };

        let result = generate_teaching(&ctx);

        assert!(result.success, "Teaching explanation must succeed for procedural with docs");
        assert!(!result.explanation.is_empty(), "Explanation must not be empty");
        assert!(!result.citations.is_empty(), "Must include at least one citation");
        assert!(
            result.citations.iter().any(|c| c.contains("man") || c.contains("Arch Wiki")),
            "Citation must reference docs (man page or Arch Wiki)"
        );
    }

    /// v0.3.29: Mandatory Milestone 4 test
    /// Probe-only questions do NOT require docs
    #[test]
    fn test_teaching_mode_probe_only_allows_probe_explanation_no_docs() {
        let ctx = TeachingContext {
            question_type: QuestionType::ProbeOnly,
            probes: vec![ProbeResult {
                command: "free -h".to_string(),
                output_summary: "Mem: 7.2G available".to_string(),
                success: true,
            }],
            doc_citations: vec![], // Explicitly no docs
            ..Default::default()
        };

        let result = generate_teaching(&ctx);

        assert!(result.success, "Probe-only must succeed without docs");
        assert!(
            result.citations.iter().any(|c| c.contains("free -h")),
            "Must reference the probe command"
        );
        assert!(
            !result.citations.iter().any(|c| c.contains("man") || c.contains("Arch Wiki")),
            "Probe-only should not require doc citations"
        );
    }

    #[test]
    fn test_citation_display_formats() {
        let probe = CitationSource::Probe {
            command: "df -h".to_string(),
            summary: "50G available".to_string(),
        };
        assert!(probe.display().contains("probe: df -h"));

        let man = CitationSource::ManPage {
            command: "systemctl".to_string(),
            section: Some("1".to_string()),
        };
        assert_eq!(man.display(), "[man systemctl(1)]");

        let wiki = CitationSource::ArchWiki {
            article: "Systemd".to_string(),
            section: Some("Timers".to_string()),
        };
        assert_eq!(wiki.display(), "[Arch Wiki: Systemd - Timers]");
    }
}
