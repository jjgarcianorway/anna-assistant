//! Answer verification through ClaimGate with evidence formatting.

use anna_shared::claim_gate::{ClaimGate, ClaimVerifier, EvidenceType};
use anna_shared::docs;
use tracing::info;

/// Result of ClaimGate verification
pub struct VerificationResult {
    /// The answer text (may have [unverified] markers)
    pub answer: String,
    /// Evidence line to append (formatted)
    pub evidence_line: String,
    /// Doc citations for teaching mode
    pub doc_citations: Vec<String>,
}

/// Format evidence line for display (with doc citations).
/// Mark empty output probes.
pub fn format_evidence_line(
    commands: &[(String, String, i32)],
    doc_citations: &[String],
    debug_mode: bool,
) -> String {
    if commands.is_empty() && doc_citations.is_empty() {
        return String::new();
    }

    if debug_mode {
        // Verbose format with exit codes and citations
        let mut lines = vec!["Evidence:".to_string()];
        for (cmd, output, exit_code) in commands {
            let status = if *exit_code == 0 {
                if output.trim().is_empty() {
                    "exit 0, empty".to_string()
                } else {
                    format!("exit {}", exit_code)
                }
            } else {
                format!("exit {}", exit_code)
            };
            lines.push(format!("  [Probe: `{}` ({})]", cmd, status));
        }
        for citation in doc_citations {
            lines.push(format!("  [Doc: {}]", citation));
        }
        lines.join("\n")
    } else {
        // Concise format - probes and citations
        let mut parts: Vec<String> = commands
            .iter()
            .map(|(cmd, output, exit_code)| {
                if *exit_code == 0 && output.trim().is_empty() {
                    format!("{}[empty]", cmd)
                } else if *exit_code != 0 {
                    format!("{}[FAILED]", cmd)
                } else {
                    cmd.clone()
                }
            })
            .collect();
        parts.extend(doc_citations.iter().cloned());
        format!("Evidence: {}", parts.join(", "))
    }
}

/// Verify answer through ClaimGate with doc citations.
/// Returns verified answer text with evidence line.
pub fn verify_answer(
    answer: &str,
    question: &str,
    executed_commands: &[(String, String, i32)],
    debug_mode: bool,
) -> VerificationResult {
    use anna_shared::claim_gate::ClaimGate as CG;

    let gate = ClaimGate::new();

    // Build evidence from executed commands
    let mut evidence: Vec<EvidenceType> = executed_commands
        .iter()
        .map(|(cmd, output, exit_code)| ClaimGate::evidence_from_probe(cmd, output, *exit_code))
        .collect();

    // Search docs if question requires documentation
    let mut doc_citations = Vec::new();
    if CG::claim_requires_docs(question) {
        // Extract key terms from question
        let terms: Vec<&str> = question
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .take(5)
            .collect();
        let search_query = terms.join(" ");

        // Search local documentation
        let citations = docs::search_docs(&search_query);
        for citation in &citations {
            doc_citations.push(citation.format_short());
            evidence.push(CG::evidence_from_doc_citation(citation));
        }
    }

    // Verify the response with question context
    let verified = gate.verify_response_with_context(answer, question, &evidence);

    let verified_answer = if verified.unverified_claims.is_empty() {
        // All claims verified, return original
        answer.to_string()
    } else {
        // Some claims unverified, return marked text
        info!(
            "ClaimGate: {} claims verified, {} unverified, docs_required={}, docs_found={}",
            verified.verified_claims.len(),
            verified.unverified_claims.len(),
            verified.docs_required,
            verified.docs_found
        );
        verified.verified_text
    };

    // Format evidence line with doc citations
    let evidence_line = format_evidence_line(executed_commands, &doc_citations, debug_mode);

    VerificationResult {
        answer: verified_answer,
        evidence_line,
        doc_citations,
    }
}

/// Truncate string with ellipsis
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Find a valid UTF-8 character boundary
        let mut end = max_len;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_evidence_line_empty() {
        assert_eq!(format_evidence_line(&[], &[], false), "");
    }

    #[test]
    fn test_format_evidence_line_commands_only() {
        let cmds = vec![
            ("uname -r".to_string(), "6.7.0".to_string(), 0),
        ];
        let result = format_evidence_line(&cmds, &[], false);
        assert!(result.contains("uname -r"));
    }

    #[test]
    fn test_format_evidence_line_empty_output() {
        let cmds = vec![
            ("cmd".to_string(), "".to_string(), 0),
        ];
        let result = format_evidence_line(&cmds, &[], false);
        assert!(result.contains("[empty]"));
    }

    #[test]
    fn test_format_evidence_line_failed() {
        let cmds = vec![
            ("cmd".to_string(), "error".to_string(), 1),
        ];
        let result = format_evidence_line(&cmds, &[], false);
        assert!(result.contains("[FAILED]"));
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        assert_eq!(truncate("hello world", 5), "hello...");
    }
}
