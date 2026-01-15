//! Evidence formatting and ClaimGate verification

use anna_shared::claim_gate::{ClaimGate, ClaimVerifier, EvidenceType};
use tracing::{debug, info};

use super::types::{Finding, VerificationResult};

/// v0.3.27: Format evidence line for display with doc citations and failed probes
pub fn format_evidence_line(
    findings: &[Finding],
    doc_citations: &[String],
    failed_probes: &[String],
    debug_mode: bool,
) -> String {
    // v0.3.27: Always emit Evidence line if there are findings or citations
    // If all probes failed, say so explicitly
    if findings.is_empty() && doc_citations.is_empty() {
        return String::new();
    }

    let successful_findings: Vec<&Finding> = findings.iter().filter(|f| f.success).collect();
    let failed_findings: Vec<&Finding> = findings.iter().filter(|f| !f.success).collect();

    // If ALL probes failed, emit explicit failure notice
    if !findings.is_empty() && successful_findings.is_empty() && doc_citations.is_empty() {
        return format!(
            "Evidence: [ALL PROBES FAILED: {}]",
            failed_probes.join(", ")
        );
    }

    if debug_mode {
        format_verbose(findings, doc_citations, failed_probes)
    } else {
        format_concise(&successful_findings, &failed_findings, doc_citations)
    }
}

/// Verbose format with exit codes and doc citations
fn format_verbose(
    findings: &[Finding],
    doc_citations: &[String],
    failed_probes: &[String],
) -> String {
    let mut lines = vec!["Evidence:".to_string()];
    for f in findings {
        // v0.3.28: Phase 3 F15 - mark empty output on success
        let status = if f.success {
            if f.output.trim().is_empty() { "OK, empty" } else { "OK" }
        } else {
            "FAILED"
        };
        lines.push(format!(
            "  [Probe: `{}` ({})]",
            f.command, status
        ));
    }
    for cite in doc_citations {
        lines.push(format!("  {}", cite));
    }
    if !failed_probes.is_empty() {
        lines.push(format!("  [Failed probes: {}]", failed_probes.join(", ")));
    }
    lines.join("\n")
}

/// Concise format - successful probes and doc citations
fn format_concise(
    successful_findings: &[&Finding],
    failed_findings: &[&Finding],
    doc_citations: &[String],
) -> String {
    // v0.3.27: Mark failed probes explicitly
    // v0.3.28: Phase 3 F15 - mark empty output probes
    let mut parts: Vec<String> = successful_findings
        .iter()
        .map(|f| {
            if f.output.trim().is_empty() {
                format!("{}[empty]", f.command)
            } else {
                f.command.clone()
            }
        })
        .collect();
    parts.extend(doc_citations.iter().cloned());

    if !failed_findings.is_empty() {
        let failed_cmds: Vec<String> = failed_findings
            .iter()
            .map(|f| format!("{}[FAILED]", f.command))
            .collect();
        parts.extend(failed_cmds);
    }

    if parts.is_empty() {
        "Evidence: [no successful probes]".to_string()
    } else {
        format!("Evidence: {}", parts.join(", "))
    }
}

/// v0.3.27: Verify answer through ClaimGate with question context for doc requirements
pub fn verify_answer(
    answer: &str,
    question: &str,
    findings: &[Finding],
    debug_mode: bool,
) -> VerificationResult {
    let gate = ClaimGate::new();

    // Build probe evidence from findings
    let mut evidence: Vec<EvidenceType> = findings
        .iter()
        .map(|f| {
            ClaimGate::evidence_from_probe(
                &f.command,
                &f.output,
                if f.success { 0 } else { 1 },
            )
        })
        .collect();

    // v0.3.26: If docs are required, search for relevant documentation
    if ClaimGate::claim_requires_docs(question) {
        // Search local docs
        let doc_citations = anna_shared::docs::search_docs(question);
        for cite in &doc_citations {
            evidence.push(ClaimGate::evidence_from_doc_citation(cite));
        }
        debug!("Found {} doc citations for question", doc_citations.len());
    }

    // Verify the response with question context
    let verified = gate.verify_response_with_context(answer, question, &evidence);

    let verified_count = verified.verified_claims.len();
    let unverified_count = verified.unverified_claims.len();

    // v0.3.27: Log blocking information
    if verified.claims_blocked {
        info!(
            "ClaimGate: BLOCKED {} unverified claims, {} verified, probes_failed={}",
            unverified_count, verified_count, verified.probes_failed
        );
        for reason in &verified.block_reasons {
            debug!("Block reason: {}", reason);
        }
    } else if !verified.unverified_claims.is_empty() {
        info!(
            "ClaimGate: {} claims verified, {} unverified, docs_required={}, docs_found={}",
            verified_count, unverified_count, verified.docs_required, verified.docs_found
        );
    }

    // v0.3.27: Format evidence line with failed probes info
    let evidence_line = format_evidence_line(
        findings,
        &verified.doc_citations,
        &verified.failed_probes,
        debug_mode,
    );

    VerificationResult {
        // v0.3.27: Use verified_text which has blocked claims replaced with uncertainty
        answer: verified.verified_text,
        evidence_line,
        needs_investigation: verified.needs_investigation,
        suggested_probes: verified.suggested_probes,
        verified_count,
        unverified_count,
    }
}
