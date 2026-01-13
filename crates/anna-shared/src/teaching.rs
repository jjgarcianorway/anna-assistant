//! Teaching Mode - Explains why actions were taken with citations.
//!
//! v0.3.29: Initial implementation for Milestone 4
//!
//! Teaching mode provides concise explanations for:
//! - Probe-only questions: explains which probe was run and why
//! - Procedural questions: explains meaning with doc citations
//! - Risky actions: explains why considered risky and why sandboxing was used

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

/// Generate a teaching explanation based on context
///
/// Rules:
/// - ProbeOnly: explain which probe answers the question (no docs required)
/// - Procedural: MUST have doc citation for semantic claims
/// - RiskyAction: MUST explain why risky and cite docs/experiment
pub fn generate_teaching(ctx: &TeachingContext) -> TeachingExplanation {
    match ctx.question_type {
        QuestionType::ProbeOnly => generate_probe_explanation(ctx),
        QuestionType::Procedural => generate_procedural_explanation(ctx),
        QuestionType::RiskyAction => generate_risky_explanation(ctx),
    }
}

/// Generate explanation for probe-only questions
/// Does NOT require doc citations - probe output is sufficient
fn generate_probe_explanation(ctx: &TeachingContext) -> TeachingExplanation {
    if ctx.probes.is_empty() {
        return TeachingExplanation::cannot_explain("No probes were executed");
    }

    let probe = &ctx.probes[0];
    let citation = CitationSource::Probe {
        command: probe.command.clone(),
        summary: probe.output_summary.clone(),
    };

    let explanation = format!(
        "This answer comes from running `{}`, which directly reports the requested information.",
        probe.command
    );

    TeachingExplanation::success(explanation, vec![citation.display()])
}

/// Generate explanation for procedural/semantic questions
/// MUST have doc citation - cannot improvise semantics
fn generate_procedural_explanation(ctx: &TeachingContext) -> TeachingExplanation {
    // Check for doc citations first - required for procedural claims
    let doc_citations: Vec<_> = ctx.doc_citations.iter()
        .filter(|c| matches!(c, CitationSource::ManPage { .. } | CitationSource::ArchWiki { .. } | CitationSource::HelpOutput { .. }))
        .collect();

    if doc_citations.is_empty() {
        return TeachingExplanation::cannot_explain(
            "Procedural question requires documentation citation but none available"
        );
    }

    let mut citations: Vec<String> = doc_citations.iter().map(|c| c.display()).collect();

    // Build explanation referencing the docs
    let doc_ref = &doc_citations[0];
    let explanation = match doc_ref {
        CitationSource::ManPage { command, .. } => {
            format!(
                "The behavior is documented in the {} manual page. {}",
                command,
                doc_ref.display()
            )
        }
        CitationSource::ArchWiki { article, .. } => {
            format!(
                "This is explained in the Arch Wiki article on {}. {}",
                article,
                doc_ref.display()
            )
        }
        CitationSource::HelpOutput { command } => {
            format!(
                "The `{} --help` output describes this functionality. {}",
                command,
                doc_ref.display()
            )
        }
        _ => {
            format!("See the referenced documentation. {}", doc_ref.display())
        }
    };

    // Add probe citations if present
    for probe in &ctx.probes {
        citations.push(CitationSource::Probe {
            command: probe.command.clone(),
            summary: probe.output_summary.clone(),
        }.display());
    }

    TeachingExplanation::success(explanation, citations)
}

/// Generate explanation for risky actions
/// MUST explain why risky and cite docs/experiment results
fn generate_risky_explanation(ctx: &TeachingContext) -> TeachingExplanation {
    let mut citations = Vec::new();
    let mut explanation_parts = Vec::new();

    // Explain why risky (principle, not score)
    if let Some(reason) = &ctx.risk_reason {
        explanation_parts.push(format!("This action is considered risky because {}.", reason));
    } else {
        explanation_parts.push("This action modifies system state and was tested first.".to_string());
    }

    // Add experiment results if available
    if !ctx.experiments.is_empty() {
        let exp = &ctx.experiments[0];
        let exp_citation = CitationSource::Experiment {
            name: exp.name.clone(),
            expected: exp.expected.clone(),
            actual: exp.actual.clone(),
        };
        citations.push(exp_citation.display());
        explanation_parts.push(format!(
            "A {} sandbox was used to verify the outcome before applying.",
            exp.sandbox_type
        ));
    }

    // Add doc citations
    for doc in &ctx.doc_citations {
        if matches!(doc, CitationSource::ManPage { .. } | CitationSource::ArchWiki { .. } | CitationSource::HelpOutput { .. }) {
            citations.push(doc.display());
        }
    }

    // If no citations at all, cannot explain
    if citations.is_empty() {
        return TeachingExplanation::cannot_explain(
            "Risky action requires documentation or experiment citation but none available"
        );
    }

    let explanation = explanation_parts.join(" ");
    TeachingExplanation::success(explanation, citations)
}

/// Classify a question to determine teaching approach
pub fn classify_question(question: &str) -> QuestionType {
    let q = question.to_lowercase();

    // Procedural keywords
    let procedural_patterns = [
        "what does", "what is", "how does", "explain", "what happens",
        "meaning of", "difference between", "why does", "purpose of",
    ];

    for pattern in procedural_patterns {
        if q.contains(pattern) {
            return QuestionType::Procedural;
        }
    }

    // Default to probe-only for factual queries
    QuestionType::ProbeOnly
}

/// Format teaching block for display
/// Returns None if teaching mode disabled or explanation failed
pub fn format_teaching_block(
    explanation: &TeachingExplanation,
    teaching_enabled: bool,
) -> Option<String> {
    if !teaching_enabled {
        return None;
    }

    if !explanation.success {
        return Some(format!(
            "\nWhy this works: {}",
            explanation.explanation
        ));
    }

    let mut output = format!("\nWhy this works: {}", explanation.explanation);
    if !explanation.citations.is_empty() {
        output.push_str("\nSources: ");
        output.push_str(&explanation.citations.join(", "));
    }

    Some(output)
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
        // Question: "what does systemctl mask do?"
        // Must include a citation like [man systemctl(1)] or [Arch Wiki: Systemd]

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

        // Must succeed
        assert!(result.success, "Teaching explanation must succeed for procedural with docs");

        // Must have "Why this works" style explanation
        assert!(!result.explanation.is_empty(), "Explanation must not be empty");

        // Must include at least one citation
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
        // Question: "how much free RAM?"
        // Teaching block should reference probe, NOT require docs

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

        // Must succeed even without docs
        assert!(result.success, "Probe-only must succeed without docs");

        // Must reference the probe
        assert!(
            result.citations.iter().any(|c| c.contains("free -h")),
            "Must reference the probe command"
        );

        // Should NOT require doc citations
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
