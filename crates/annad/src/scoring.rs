//! Deterministic reliability scoring.
//!
//! Calculates reliability score from concrete signals, not vibes.
//! GUARD integration for evidence-based invention detection.

use anna_shared::claims::extract_claims;
use anna_shared::grounding::ParsedEvidence;
use anna_shared::guard::{run_guard, GuardReport};
use anna_shared::rpc::ProbeResult;

/// Minimum translator confidence to be considered "confident"
#[allow(dead_code)]
pub const CONFIDENCE_THRESHOLD: f32 = 0.7;

/// Check if answer references probe data or hardware info
pub fn check_answer_grounded(answer: &str, probe_results: &[ProbeResult]) -> bool {
    let answer_lower = answer.to_lowercase();

    // If there are probe results, check if answer references them
    if !probe_results.is_empty() {
        // Look for specific data patterns from probes
        for probe in probe_results {
            if probe.exit_code == 0 && !probe.stdout.is_empty() {
                // Check if answer contains data from probe output
                // Look for numbers, percentages, process names, etc.
                let probe_lines: Vec<&str> = probe.stdout.lines().take(5).collect();
                for line in probe_lines {
                    // Extract potential data points (numbers, percentages)
                    let words: Vec<&str> = line.split_whitespace().collect();
                    for word in words {
                        // If answer contains a specific value from probe
                        if word.len() > 2
                            && (word.parse::<f64>().is_ok()
                                || word.ends_with('%')
                                || word.ends_with('G')
                                || word.ends_with('M'))
                            && answer_lower.contains(&word.to_lowercase())
                        {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // Check for grounding indicators in the answer
    let grounding_phrases = [
        "according to",
        "the output shows",
        "as shown",
        "currently",
        "the data indicates",
        "probe results",
        "system reports",
        "output:",
        "result:",
    ];

    grounding_phrases
        .iter()
        .any(|phrase| answer_lower.contains(phrase))
}

/// Check if answer appears to invent facts (legacy heuristic).
/// Kept for backwards compatibility; prefer check_no_invention_guard for new code.
pub fn check_no_invention(answer: &str) -> bool {
    let answer_lower = answer.to_lowercase();

    // Red flags for invention
    let invention_indicators = [
        "i don't have access",
        "i cannot determine",
        "i would need to",
        "typically",
        "usually",
        "generally",
        "might be",
        "could be",
        "probably",
        "i assume",
        "i believe",
        "it's likely",
        "most likely",
    ];

    // If many invention indicators present, flag it
    let invention_count = invention_indicators
        .iter()
        .filter(|ind| answer_lower.contains(*ind))
        .count();

    // Allow one hedging word, but flag if multiple
    invention_count <= 1
}

// === GUARD-based invention detection ===

/// Run GUARD verification and return report.
/// Uses claim extraction from ANCHOR phase.
pub fn run_guard_check(
    answer: &str,
    evidence: &ParsedEvidence,
    evidence_required: bool,
) -> GuardReport {
    let claims = extract_claims(answer);
    run_guard(&claims, evidence, evidence_required)
}

/// Check for invention using GUARD (evidence-based).
/// Returns true if NO invention detected (same semantics as check_no_invention).
pub fn check_no_invention_guard(
    answer: &str,
    evidence: &ParsedEvidence,
    evidence_required: bool,
) -> bool {
    let report = run_guard_check(answer, evidence, evidence_required);
    !report.invention_detected
}


#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::rpc::ReliabilitySignals;

    fn make_probe_result(cmd: &str, exit_code: i32, stdout: &str) -> ProbeResult {
        ProbeResult {
            command: cmd.to_string(),
            exit_code,
            stdout: stdout.to_string(),
            stderr: String::new(),
            timing_ms: 100,
        }
    }

    #[test]
    fn test_answer_grounded_with_data() {
        let probes = vec![make_probe_result("free -h", 0, "total: 16G")];
        assert!(check_answer_grounded("Your system has 16G of RAM", &probes));
    }

    #[test]
    fn test_answer_grounded_with_phrases() {
        let probes = vec![];
        assert!(check_answer_grounded(
            "According to the system data...",
            &probes
        ));
        assert!(check_answer_grounded("The output shows that...", &probes));
    }

    #[test]
    fn test_no_invention_clean() {
        assert!(check_no_invention("The CPU usage is 50%"));
    }

    #[test]
    fn test_no_invention_flagged() {
        assert!(!check_no_invention(
            "I would need to check, and it's probably around 50%, typically systems use..."
        ));
    }

    #[test]
    fn test_reliability_score_calculation() {
        let signals = ReliabilitySignals {
            translator_confident: true,
            probe_coverage: true,
            answer_grounded: true,
            no_invention: true,
            clarification_not_needed: true,
        };
        assert_eq!(signals.score(), 100);

        let signals = ReliabilitySignals {
            translator_confident: false,
            probe_coverage: false,
            answer_grounded: false,
            no_invention: false,
            clarification_not_needed: false,
        };
        assert_eq!(signals.score(), 0);

        let signals = ReliabilitySignals {
            translator_confident: true,
            probe_coverage: true,
            answer_grounded: false,
            no_invention: true,
            clarification_not_needed: true,
        };
        assert_eq!(signals.score(), 80);
    }

    // === GUARD integration tests ===

    use anna_shared::parsers::{DiskUsage, ServiceState, ServiceStatus};

    #[test]
    fn test_guard_verified_no_invention() {
        let answer = "nginx is running";
        let evidence = ParsedEvidence {
            services: vec![ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Running,
                description: None,
            }],
            ..Default::default()
        };

        assert!(check_no_invention_guard(answer, &evidence, true));
    }

    #[test]
    fn test_guard_contradiction_flags_invention() {
        let answer = "nginx is running";
        let evidence = ParsedEvidence {
            services: vec![ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Failed, // Contradiction!
                description: None,
            }],
            ..Default::default()
        };

        // Contradiction flags invention regardless of evidence_required
        assert!(!check_no_invention_guard(answer, &evidence, false));
        assert!(!check_no_invention_guard(answer, &evidence, true));
    }

    #[test]
    fn test_guard_unverifiable_with_evidence_required() {
        let answer = "firefox uses 4294967296B";
        let evidence = ParsedEvidence::default(); // No evidence

        // Unverifiable only flags when evidence_required
        assert!(check_no_invention_guard(answer, &evidence, false));
        assert!(!check_no_invention_guard(answer, &evidence, true));
    }

    #[test]
    fn test_guard_report_details() {
        let answer = "root is 85% full and nginx is running";
        let evidence = ParsedEvidence {
            disks: vec![DiskUsage {
                filesystem: "/dev/sda1".to_string(),
                mount: "/".to_string(),
                size_bytes: 100_000_000_000,
                used_bytes: 85_000_000_000,
                available_bytes: 15_000_000_000,
                percent_used: 85,
            }],
            services: vec![ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Running,
                description: None,
            }],
            ..Default::default()
        };

        let report = run_guard_check(answer, &evidence, true);
        assert_eq!(report.total_specific_claims, 2);
        assert_eq!(report.contradictions, 0);
        assert_eq!(report.unverifiable_specifics, 0);
        assert!(!report.invention_detected);
    }
}
