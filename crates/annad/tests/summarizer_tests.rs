//! Tests for probe output summarizer.
//!
//! Verifies that probe outputs are correctly truncated to <= 15 lines.

use anna_shared::rpc::ProbeResult;

/// Maximum lines per summarized probe output
const MAX_SUMMARY_LINES: usize = 15;

fn make_probe(lines: usize) -> ProbeResult {
    let stdout = (0..lines)
        .map(|i| format!("line {}: some content here", i))
        .collect::<Vec<_>>()
        .join("\n");
    ProbeResult {
        command: "test".to_string(),
        exit_code: 0,
        stdout,
        stderr: String::new(),
        timing_ms: 100,
    }
}

fn summarize(probe: &ProbeResult) -> (String, bool) {
    let lines: Vec<&str> = probe.stdout.lines().collect();
    let total = lines.len();

    if total <= MAX_SUMMARY_LINES {
        (probe.stdout.clone(), false)
    } else {
        let kept: Vec<&str> = lines.iter().take(MAX_SUMMARY_LINES - 1).copied().collect();
        let omitted = total - (MAX_SUMMARY_LINES - 1);
        let summary = format!("{}\n... ({} more lines)", kept.join("\n"), omitted);
        (summary, true)
    }
}

#[test]
fn test_short_probe_not_truncated() {
    let probe = make_probe(5);
    let (summary, truncated) = summarize(&probe);
    assert!(!truncated);
    assert_eq!(summary.lines().count(), 5);
}

#[test]
fn test_exact_limit_not_truncated() {
    let probe = make_probe(15);
    let (summary, truncated) = summarize(&probe);
    assert!(!truncated);
    assert_eq!(summary.lines().count(), 15);
}

#[test]
fn test_over_limit_truncated() {
    let probe = make_probe(50);
    let (summary, truncated) = summarize(&probe);
    assert!(truncated);
    assert!(summary.lines().count() <= MAX_SUMMARY_LINES);
    assert!(summary.contains("more lines"));
}

#[test]
fn test_large_probe_truncated() {
    let probe = make_probe(200);
    let (summary, truncated) = summarize(&probe);
    assert!(truncated);
    assert!(summary.lines().count() <= MAX_SUMMARY_LINES);
    // Should mention how many lines were omitted
    assert!(summary.contains("more lines"));
}

#[test]
fn test_empty_probe() {
    let probe = ProbeResult {
        command: "test".to_string(),
        exit_code: 0,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 50,
    };
    let (summary, truncated) = summarize(&probe);
    assert!(!truncated);
    assert!(summary.is_empty());
}

#[test]
fn test_single_line_probe() {
    let probe = make_probe(1);
    let (summary, truncated) = summarize(&probe);
    assert!(!truncated);
    assert_eq!(summary.lines().count(), 1);
}
