//! Probe output summarizer.
//!
//! Compacts probe outputs to <= 15 lines for specialist consumption.
//! Ensures LLM doesn't receive overwhelming raw data.

use anna_shared::rpc::ProbeResult;

/// Maximum lines per summarized probe output
pub const MAX_SUMMARY_LINES: usize = 15;

/// Summarized probe output
#[derive(Debug, Clone)]
pub struct SummarizedProbe {
    pub command: String,
    pub exit_code: i32,
    pub summary: String,
    #[allow(dead_code)]
    pub line_count: usize,
    #[allow(dead_code)]
    pub was_truncated: bool,
}

/// Summarize a single probe result to <= MAX_SUMMARY_LINES
pub fn summarize_probe(probe: &ProbeResult) -> SummarizedProbe {
    let lines: Vec<&str> = probe.stdout.lines().collect();
    let total_lines = lines.len();

    let (summary, was_truncated) = if total_lines <= MAX_SUMMARY_LINES {
        (probe.stdout.clone(), false)
    } else {
        // Take first lines, add truncation notice
        let kept: Vec<&str> = lines.iter().take(MAX_SUMMARY_LINES - 1).copied().collect();
        let omitted = total_lines - (MAX_SUMMARY_LINES - 1);
        let mut s = kept.join("\n");
        s.push_str(&format!("\n... ({} more lines)", omitted));
        (s, true)
    };

    SummarizedProbe {
        command: probe.command.clone(),
        exit_code: probe.exit_code,
        summary,
        line_count: total_lines.min(MAX_SUMMARY_LINES),
        was_truncated,
    }
}

/// Summarize all probe results
#[allow(dead_code)]
pub fn summarize_probes(probes: &[ProbeResult]) -> Vec<SummarizedProbe> {
    probes.iter().map(summarize_probe).collect()
}

/// Build compact probe context for specialist (text format)
pub fn build_probe_context(probes: &[ProbeResult]) -> String {
    if probes.is_empty() {
        return String::new();
    }

    let mut context = String::new();
    for probe in probes {
        let summarized = summarize_probe(probe);
        if summarized.exit_code == 0 {
            context.push_str(&format!("--- {} ---\n", summarized.command));
            context.push_str(&summarized.summary);
            context.push_str("\n\n");
        } else {
            context.push_str(&format!(
                "--- {} (exit {}) ---\n",
                summarized.command, summarized.exit_code
            ));
            if !probe.stderr.is_empty() {
                let stderr_lines: Vec<&str> = probe.stderr.lines().take(3).collect();
                context.push_str(&stderr_lines.join("\n"));
                context.push('\n');
            }
            context.push('\n');
        }
    }

    context
}

/// Calculate total summarized size in bytes
#[allow(dead_code)]
pub fn total_summary_size(probes: &[ProbeResult]) -> usize {
    probes
        .iter()
        .map(|p| summarize_probe(p).summary.len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_probe(stdout_lines: usize) -> ProbeResult {
        let stdout = (0..stdout_lines)
            .map(|i| format!("line {}", i))
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

    #[test]
    fn test_short_probe_not_truncated() {
        let probe = make_probe(5);
        let summary = summarize_probe(&probe);
        assert!(!summary.was_truncated);
        assert_eq!(summary.line_count, 5);
    }

    #[test]
    fn test_long_probe_truncated() {
        let probe = make_probe(50);
        let summary = summarize_probe(&probe);
        assert!(summary.was_truncated);
        assert!(summary.summary.contains("more lines"));
        assert!(summary.summary.lines().count() <= MAX_SUMMARY_LINES);
    }

    #[test]
    fn test_exact_limit_not_truncated() {
        let probe = make_probe(MAX_SUMMARY_LINES);
        let summary = summarize_probe(&probe);
        assert!(!summary.was_truncated);
    }

    #[test]
    fn test_build_probe_context() {
        let probes = vec![
            ProbeResult {
                command: "ps aux".to_string(),
                exit_code: 0,
                stdout: "USER PID\nroot 1\nroot 2".to_string(),
                stderr: String::new(),
                timing_ms: 50,
            },
            ProbeResult {
                command: "df -h".to_string(),
                exit_code: 0,
                stdout: "Filesystem Size\n/dev/sda1 100G".to_string(),
                stderr: String::new(),
                timing_ms: 30,
            },
        ];

        let context = build_probe_context(&probes);
        assert!(context.contains("ps aux"));
        assert!(context.contains("df -h"));
        assert!(context.contains("USER PID"));
    }

    #[test]
    fn test_failed_probe_in_context() {
        let probes = vec![ProbeResult {
            command: "failing_cmd".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: "command not found".to_string(),
            timing_ms: 10,
        }];

        let context = build_probe_context(&probes);
        assert!(context.contains("exit 1"));
        assert!(context.contains("command not found"));
    }
}
