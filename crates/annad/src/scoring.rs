//! Deterministic reliability scoring.
//!
//! Calculates reliability score from concrete signals, not vibes.

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

/// Check if answer appears to invent facts
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
}
