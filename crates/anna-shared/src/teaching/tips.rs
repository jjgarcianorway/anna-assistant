//! Teaching mode explanation generators.

use super::{
    CitationSource, ExperimentSummary, ProbeResult, QuestionType, TeachingContext,
    TeachingExplanation,
};

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
    let doc_citations: Vec<_> = ctx
        .doc_citations
        .iter()
        .filter(|c| {
            matches!(
                c,
                CitationSource::ManPage { .. }
                    | CitationSource::ArchWiki { .. }
                    | CitationSource::HelpOutput { .. }
            )
        })
        .collect();

    if doc_citations.is_empty() {
        return TeachingExplanation::cannot_explain(
            "Procedural question requires documentation citation but none available",
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
        citations.push(
            CitationSource::Probe {
                command: probe.command.clone(),
                summary: probe.output_summary.clone(),
            }
            .display(),
        );
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
        explanation_parts
            .push("This action modifies system state and was tested first.".to_string());
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
        if matches!(
            doc,
            CitationSource::ManPage { .. }
                | CitationSource::ArchWiki { .. }
                | CitationSource::HelpOutput { .. }
        ) {
            citations.push(doc.display());
        }
    }

    // If no citations at all, cannot explain
    if citations.is_empty() {
        return TeachingExplanation::cannot_explain(
            "Risky action requires documentation or experiment citation but none available",
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
        "what does",
        "what is",
        "how does",
        "explain",
        "what happens",
        "meaning of",
        "difference between",
        "why does",
        "purpose of",
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
        return Some(format!("\nWhy this works: {}", explanation.explanation));
    }

    let mut output = format!("\nWhy this works: {}", explanation.explanation);
    if !explanation.citations.is_empty() {
        output.push_str("\nSources: ");
        output.push_str(&explanation.citations.join(", "));
    }

    Some(output)
}
