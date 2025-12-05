//! Deterministic triage answer generator (v0.0.35 FAST PATH).
//!
//! Generates system triage answers from journal errors/warnings, failed units, and boot time.
//! No LLM required - purely deterministic based on evidence.
//!
//! v0.0.35: Uses anna_shared::parsers for consistent parsing across crates.

use anna_shared::parsers::{
    parse_journalctl_priority, parse_boot_time, parse_journal_failed_units,
    JournalSummary, BootTimeInfo, JournalFailedUnit,
};
use anna_shared::rpc::ProbeResult;
use crate::parsers::find_probe;
use crate::deterministic::DeterministicResult;

/// System triage evidence collected from probes
#[derive(Debug, Default)]
pub struct TriageEvidence {
    pub errors: JournalSummary,
    pub warnings: JournalSummary,
    pub failed_units: Vec<JournalFailedUnit>,
    pub boot_time: Option<BootTimeInfo>,
}

impl TriageEvidence {
    /// Check if system has no critical issues
    pub fn is_healthy(&self) -> bool {
        self.errors.count_total == 0 && self.failed_units.is_empty()
    }

    /// Count total issues
    pub fn issue_count(&self) -> usize {
        self.errors.count_total as usize + self.failed_units.len()
    }

    /// List evidence kinds used
    pub fn evidence_kinds(&self) -> Vec<&str> {
        let mut kinds = Vec::new();
        if self.errors.count_total > 0 || self.warnings.count_total > 0 {
            kinds.push("journal");
        }
        if !self.failed_units.is_empty() {
            kinds.push("failed_units");
        }
        if self.boot_time.is_some() {
            kinds.push("boot_time");
        }
        if kinds.is_empty() {
            kinds.push("system_state");
        }
        kinds
    }
}

/// Collect triage evidence from probe results
pub fn collect_triage_evidence(probes: &[ProbeResult]) -> TriageEvidence {
    let mut evidence = TriageEvidence::default();

    // Parse journal errors (priority 3 = err)
    if let Some(probe) = find_probe(probes, "journalctl -p 3") {
        evidence.errors = parse_journalctl_priority(&probe.stdout);
    }

    // Parse journal warnings (priority 4 = warn)
    if let Some(probe) = find_probe(probes, "journalctl -p 4") {
        evidence.warnings = parse_journalctl_priority(&probe.stdout);
    }

    // Parse failed units (v0.0.35: use journal parser for consistency)
    if let Some(probe) = find_probe(probes, "systemctl --failed") {
        evidence.failed_units = parse_journal_failed_units(&probe.stdout);
    }

    // Parse boot time (v0.0.35)
    if let Some(probe) = find_probe(probes, "systemd-analyze") {
        let boot_info = parse_boot_time(&probe.stdout);
        if !boot_info.raw_line.is_empty() {
            evidence.boot_time = Some(boot_info);
        }
    }

    evidence
}

/// Generate deterministic triage answer from evidence
pub fn generate_triage_answer(probes: &[ProbeResult]) -> Option<DeterministicResult> {
    let evidence = collect_triage_evidence(probes);

    let answer = format_triage_answer(&evidence);
    let count = evidence.errors.count_total as usize
        + evidence.warnings.count_total as usize
        + evidence.failed_units.len();

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: if count == 0 { 1 } else { count },
        route_class: "system_triage".to_string(),
    })
}

/// Format triage answer according to v0.0.35 rules:
/// - If no errors, no failed units: "No critical issues detected." Then if warnings > 0,
///   include "Warnings detected: N" and top 3 keys.
/// - If errors > 0: Output "Critical issues detected: N errors." Include top 3 keys with counts.
/// - If failed units exist: Include list of failed unit names (max 5).
/// - If boot_time present: Show as informational line.
/// - Do not ask the user to rephrase.
/// - Output must include "Evidence:" line listing which evidence kinds were used.
fn format_triage_answer(evidence: &TriageEvidence) -> String {
    let mut lines = Vec::new();

    if evidence.is_healthy() {
        // No critical issues
        lines.push("**No critical issues detected.**".to_string());

        // Show warnings if present (top 3 keys)
        if evidence.warnings.count_total > 0 {
            lines.push(String::new());
            lines.push(format!("Warnings detected: {}", evidence.warnings.count_total));
            for item in evidence.warnings.top.iter().take(3) {
                lines.push(format!("- `{}`: {} occurrences", item.key, item.count));
            }
        }
    } else {
        // Critical issues found
        if evidence.errors.count_total > 0 {
            lines.push(format!(
                "**Critical issues detected: {} errors.**",
                evidence.errors.count_total
            ));
            for item in evidence.errors.top.iter().take(3) {
                lines.push(format!("- `{}`: {} error(s)", item.key, item.count));
            }
            lines.push(String::new());
        }

        if !evidence.failed_units.is_empty() {
            lines.push(format!(
                "**{} Failed Service(s)**:",
                evidence.failed_units.len()
            ));
            for unit in evidence.failed_units.iter().take(5) {
                lines.push(format!("- `{}` ({})", unit.name, unit.active_state));
            }
            if evidence.failed_units.len() > 5 {
                lines.push(format!("  ...and {} more", evidence.failed_units.len() - 5));
            }
        }
    }

    // Boot time (informational only)
    if let Some(ref boot) = evidence.boot_time {
        lines.push(String::new());
        if let Some(total_secs) = boot.total_secs() {
            lines.push(format!("Boot time: {:.1}s", total_secs));
        } else if !boot.raw_line.is_empty() {
            lines.push(format!("Boot: {}", boot.raw_line));
        }
    }

    // Evidence summary (always include for auditability)
    lines.push(String::new());
    lines.push("---".to_string());
    let kinds = evidence.evidence_kinds().join(", ");
    lines.push(format!(
        "*Evidence: {} errors, {} warnings, {} failed units | Sources: [{}]*",
        evidence.errors.count_total,
        evidence.warnings.count_total,
        evidence.failed_units.len(),
        kinds
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
            mock_probe("systemctl --failed --no-pager", "0 loaded units listed."),
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
            mock_probe("systemctl --failed --no-pager", "0 loaded units listed."),
        ];

        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("Critical issues"));
        assert!(result.answer.contains("systemd"));
    }

    #[test]
    fn test_failed_units() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager", ""),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed --no-pager",
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
            mock_probe("systemctl --failed --no-pager", "0 loaded units listed."),
        ];

        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("No critical issues"));
        assert!(result.answer.contains("Warnings detected"));
    }

    #[test]
    fn test_boot_time_shown() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager", ""),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed --no-pager", "0 loaded units listed."),
            mock_probe("systemd-analyze", "Startup finished in 2.5s (kernel) + 5.3s (userspace) = 7.8s"),
        ];

        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("No critical issues"));
        assert!(result.answer.contains("Boot time"));
    }

    #[test]
    fn test_evidence_line_present() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager", ""),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed --no-pager", "0 loaded units listed."),
        ];

        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("Evidence:"));
        assert!(result.answer.contains("Sources:"));
    }

    // Golden test: deterministic output for same input
    #[test]
    fn golden_triage_deterministic() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager",
                "Dec 05 10:00:00 host nginx[1234]: connection refused"),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed --no-pager", "0 loaded units listed."),
        ];

        let result1 = generate_triage_answer(&probes).unwrap();
        let result2 = generate_triage_answer(&probes).unwrap();

        // Same input must produce identical output
        assert_eq!(result1.answer, result2.answer);
    }

    // v0.0.35: Scenario matrix tests
    #[test]
    fn test_no_errors_no_failed_no_warnings() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager", ""),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed --no-pager", "0 loaded units listed."),
        ];
        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("No critical issues"));
        assert!(!result.answer.contains("Warnings detected"));
    }

    #[test]
    fn test_warnings_only() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager", ""),
            mock_probe("journalctl -p 4 -b --no-pager",
                "Dec 05 10:00:00 host systemd[1]: warn1"),
            mock_probe("systemctl --failed --no-pager", "0 loaded units listed."),
        ];
        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("No critical issues"));
        assert!(result.answer.contains("Warnings detected: 1"));
    }

    #[test]
    fn test_errors_only() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager",
                "Dec 05 10:00:00 host kernel: error"),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed --no-pager", "0 loaded units listed."),
        ];
        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("Critical issues detected: 1 error"));
    }

    #[test]
    fn test_failed_units_only() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager", ""),
            mock_probe("journalctl -p 4 -b --no-pager", ""),
            mock_probe("systemctl --failed --no-pager",
                "● test.service loaded failed failed Test"),
        ];
        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("Failed Service"));
        assert!(result.answer.contains("test.service"));
    }

    #[test]
    fn test_mixed_errors_warnings_failed() {
        let probes = vec![
            mock_probe("journalctl -p 3 -b --no-pager",
                "Dec 05 10:00:00 host systemd[1]: error1
Dec 05 10:01:00 host systemd[1]: error2"),
            mock_probe("journalctl -p 4 -b --no-pager",
                "Dec 05 10:00:00 host kernel: warn1"),
            mock_probe("systemctl --failed --no-pager",
                "● nginx.service loaded failed failed Nginx"),
        ];
        let result = generate_triage_answer(&probes).unwrap();
        assert!(result.answer.contains("Critical issues detected: 2 error"));
        assert!(result.answer.contains("Failed Service"));
        assert!(result.answer.contains("nginx.service"));
    }
}
