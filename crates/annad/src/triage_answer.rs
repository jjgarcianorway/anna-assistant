//! Deterministic triage answer generator (v0.0.34 FAST PATH).
//!
//! Generates system triage answers from journal errors/warnings and failed units.
//! No LLM required - purely deterministic based on evidence.

use anna_shared::rpc::ProbeResult;
use crate::parsers::{find_probe, parse_journalctl, parse_failed_units, JournalSummary, FailedUnit};
use crate::deterministic::DeterministicResult;

/// System triage evidence collected from probes
#[derive(Debug, Default)]
pub struct TriageEvidence {
    pub errors: JournalSummary,
    pub warnings: JournalSummary,
    pub failed_units: Vec<FailedUnit>,
}

impl TriageEvidence {
    /// Check if system has no critical issues
    pub fn is_healthy(&self) -> bool {
        self.errors.total_count == 0 && self.failed_units.is_empty()
    }

    /// Count total issues
    pub fn issue_count(&self) -> usize {
        self.errors.total_count + self.failed_units.len()
    }
}

/// Collect triage evidence from probe results
pub fn collect_triage_evidence(probes: &[ProbeResult]) -> TriageEvidence {
    let mut evidence = TriageEvidence::default();

    // Parse journal errors (priority 3 = err)
    if let Some(probe) = find_probe(probes, "journalctl -p 3") {
        evidence.errors = parse_journalctl(&probe.stdout);
    }

    // Parse journal warnings (priority 4 = warn)
    if let Some(probe) = find_probe(probes, "journalctl -p 4") {
        evidence.warnings = parse_journalctl(&probe.stdout);
    }

    // Parse failed units
    if let Some(probe) = find_probe(probes, "systemctl --failed") {
        evidence.failed_units = parse_failed_units(&probe.stdout);
    }

    evidence
}

/// Generate deterministic triage answer from evidence
pub fn generate_triage_answer(probes: &[ProbeResult]) -> Option<DeterministicResult> {
    let evidence = collect_triage_evidence(probes);

    let answer = format_triage_answer(&evidence);
    let count = evidence.errors.total_count + evidence.warnings.total_count + evidence.failed_units.len();

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: if count == 0 { 1 } else { count },
        route_class: "system_triage".to_string(),
    })
}

/// Format triage answer according to rules:
/// - If no errors, no failed units: "No critical issues detected." + up to 2 warnings
/// - If errors exist: list top 3 with counts, then failed units
/// - Always include reliability score and evidence summary
fn format_triage_answer(evidence: &TriageEvidence) -> String {
    let mut lines = Vec::new();

    if evidence.is_healthy() {
        // No critical issues
        lines.push("**No critical issues detected.**".to_string());

        // Show up to 2 notable warnings if present
        if evidence.warnings.total_count > 0 {
            lines.push(String::new());
            lines.push(format!("**Warnings** ({} this boot):", evidence.warnings.total_count));
            for (unit, count) in evidence.warnings.by_unit.iter().take(2) {
                lines.push(format!("- `{}`: {} occurrences", unit, count));
            }
        }
    } else {
        // Critical issues found
        if !evidence.failed_units.is_empty() {
            lines.push(format!("**{} Failed Service(s)**:", evidence.failed_units.len()));
            for unit in &evidence.failed_units {
                lines.push(format!("- `{}` ({})", unit.name, unit.active_state));
            }
            lines.push(String::new());
        }

        if evidence.errors.total_count > 0 {
            lines.push(format!("**{} Journal Error(s)** this boot:", evidence.errors.total_count));
            for (unit, count) in evidence.errors.by_unit.iter().take(3) {
                lines.push(format!("- `{}`: {} error(s)", unit, count));
            }

            // Sample message if helpful
            if let Some(sample) = evidence.errors.sample_messages.first() {
                let truncated = if sample.len() > 100 {
                    format!("{}...", &sample[..100])
                } else {
                    sample.clone()
                };
                lines.push(format!("  Latest: {}", truncated));
            }
        }
    }

    // Evidence summary (always include for auditability)
    lines.push(String::new());
    lines.push("---".to_string());
    lines.push(format!(
        "*Evidence: {} errors, {} warnings, {} failed units (this boot)*",
        evidence.errors.total_count,
        evidence.warnings.total_count,
        evidence.failed_units.len()
    ));

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_probe(cmd: &str, stdout: &str) -> ProbeResult {
        ProbeResult {
            command: cmd.to_string(),
            exit_code: 0,
            stdout: stdout.to_string(),
            stderr: String::new(),
            timing_ms: 50,
        }
    }

    #[test]
    fn test_healthy_system() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager", ""),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed", "0 loaded units listed."),
        ];

        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("No critical issues"));
        assert!(result.grounded);
    }

    #[test]
    fn test_system_with_errors() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager",
                "Dec 05 10:00:00 host systemd[1]: Failed to start nginx.service
Dec 05 10:01:00 host systemd[1]: Another error
Dec 05 10:02:00 host kernel: disk error"),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed", "0 loaded units listed."),
        ];

        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("Journal Error"));
        assert!(result.answer.contains("systemd"));
    }

    #[test]
    fn test_failed_units() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager", ""),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed",
                "  UNIT                   LOAD   ACTIVE SUB    DESCRIPTION
● nginx.service         loaded failed failed Nginx Web Server"),
        ];

        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("Failed Service"));
        assert!(result.answer.contains("nginx.service"));
    }

    #[test]
    fn test_warnings_shown_when_healthy() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager", ""),
            mock_probe("journalctl -p 4 -b --no-pager",
                "Dec 05 10:00:00 host systemd[1]: Some warning
Dec 05 10:01:00 host systemd[1]: Another warning"),
            mock_probe("systemctl --failed", "0 loaded units listed."),
        ];

        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("No critical issues"));
        assert!(result.answer.contains("Warnings"));
    }

    // Golden test: deterministic output for same input
    #[test]
    fn golden_triage_deterministic() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager",
                "Dec 05 10:00:00 host nginx[1234]: connection refused"),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed", "0 loaded units listed."),
        ];

        let result1 = generate_triage_answer(&probes).unwrap();
        let result2 = generate_triage_answer(&probes).unwrap();

        // Same input must produce identical output
        assert_eq!(result1.answer, result2.answer);
    }
}
