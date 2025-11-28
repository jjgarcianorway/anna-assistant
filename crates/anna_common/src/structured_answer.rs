//! Structured Answer v0.83.0
//!
//! Schema for machine-parseable and TUI-friendly answer output.
//! Used by both the TUI renderer and ANNA_QA_MODE JSON output.

use serde::{Deserialize, Serialize};

/// Structured answer format for consistent TUI and QA output
///
/// For Green answers (>=90%):
/// - headline: Direct answer (e.g., "You have 32 threads available")
/// - details: Concrete numbers and specs
/// - evidence: Probe fields with values
///
/// For Yellow/Red answers:
/// - headline: Cautious statement (e.g., "Unable to determine with confidence")
/// - details: Explain why confidence is low
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredAnswer {
    /// Short one-line summary answering the question directly
    pub headline: String,
    /// Bullet-point details with concrete numbers/specs
    pub details: Vec<String>,
    /// Evidence summaries from probes (e.g., "cpu.info: CPU(s): 32, Model: AMD Ryzen...")
    pub evidence: Vec<String>,
    /// Reliability label: Green, Yellow, or Red
    pub reliability_label: String,
}

impl StructuredAnswer {
    /// Create a new structured answer
    pub fn new(
        headline: impl Into<String>,
        details: Vec<String>,
        evidence: Vec<String>,
        reliability_label: impl Into<String>,
    ) -> Self {
        Self {
            headline: headline.into(),
            details,
            evidence,
            reliability_label: reliability_label.into(),
        }
    }

    /// Create a Green (high confidence) answer
    pub fn green(headline: impl Into<String>, details: Vec<String>, evidence: Vec<String>) -> Self {
        Self::new(headline, details, evidence, "Green")
    }

    /// Create a Yellow (medium confidence) answer
    pub fn yellow(headline: impl Into<String>, details: Vec<String>, evidence: Vec<String>) -> Self {
        Self::new(headline, details, evidence, "Yellow")
    }

    /// Create a Red (low confidence) answer
    pub fn red(headline: impl Into<String>, details: Vec<String>, evidence: Vec<String>) -> Self {
        Self::new(headline, details, evidence, "Red")
    }

    /// Create a timeout answer
    pub fn timeout(budget_ms: u64, actual_ms: u64) -> Self {
        Self::red(
            "Timed out before completing analysis",
            vec![
                format!("Budget: {}ms, Actual: {}ms", budget_ms, actual_ms),
                "Try again or simplify the question".to_string(),
            ],
            vec![],
        )
    }

    /// Create a refusal answer
    pub fn refusal(reason: impl Into<String>) -> Self {
        Self::red(
            "Unable to answer this question",
            vec![reason.into()],
            vec![],
        )
    }

    /// Check if this is a high-confidence answer
    pub fn is_green(&self) -> bool {
        self.reliability_label == "Green"
    }

    /// Check if this is a medium-confidence answer
    pub fn is_yellow(&self) -> bool {
        self.reliability_label == "Yellow"
    }

    /// Check if this is a low-confidence answer
    pub fn is_red(&self) -> bool {
        self.reliability_label == "Red"
    }
}

/// Dialog trace for QA JSON output
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DialogTrace {
    /// Probes requested by Junior
    pub junior_plan_probes: Vec<String>,
    /// Whether Junior provided a draft answer
    pub junior_had_draft: bool,
    /// Senior's verdict (approve, fix_and_accept, refuse)
    pub senior_verdict: String,
}

impl DialogTrace {
    pub fn new(junior_plan_probes: Vec<String>, junior_had_draft: bool, senior_verdict: &str) -> Self {
        Self {
            junior_plan_probes,
            junior_had_draft,
            senior_verdict: senior_verdict.to_string(),
        }
    }
}

/// QA-mode JSON output with latency and trace data (v0.82.0)
///
/// This is the stable schema for benchmarking and trend analysis.
/// All fields must be populated for each question run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaOutput {
    /// The structured answer (flattened into output)
    #[serde(flatten)]
    pub answer: StructuredAnswer,
    /// Overall confidence score (0.0-1.0)
    pub score_overall: f64,
    /// Junior LLM latency in milliseconds
    pub junior_ms: u64,
    /// Senior LLM latency in milliseconds
    pub senior_ms: u64,
    /// Number of iterations used
    pub iterations: u32,
    /// List of probe IDs used in this answer
    pub probes_used: Vec<String>,
    /// Error kind if the answer failed, null if ok
    /// Values: null, "timeout", "llm_parse_error", "probe_failure", "refused"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<String>,
    /// Dialog trace showing the flow
    pub dialog_trace: DialogTrace,
}

impl QaOutput {
    /// Create a new QA output with all required fields
    pub fn new(
        answer: StructuredAnswer,
        score_overall: f64,
        junior_ms: u64,
        senior_ms: u64,
        iterations: u32,
        probes_used: Vec<String>,
        error_kind: Option<String>,
        dialog_trace: DialogTrace,
    ) -> Self {
        Self {
            answer,
            score_overall,
            junior_ms,
            senior_ms,
            iterations,
            probes_used,
            error_kind,
            dialog_trace,
        }
    }

    /// Create an error QA output
    pub fn error(error_kind: &str, headline: &str, details: Vec<String>) -> Self {
        Self {
            answer: StructuredAnswer::red(headline, details, vec![]),
            score_overall: 0.0,
            junior_ms: 0,
            senior_ms: 0,
            iterations: 0,
            probes_used: vec![],
            error_kind: Some(error_kind.to_string()),
            dialog_trace: DialogTrace::default(),
        }
    }

    /// Create a timeout error
    pub fn timeout(budget_ms: u64, actual_ms: u64, junior_ms: u64, senior_ms: u64) -> Self {
        Self {
            answer: StructuredAnswer::timeout(budget_ms, actual_ms),
            score_overall: 0.0,
            junior_ms,
            senior_ms,
            iterations: 0,
            probes_used: vec![],
            error_kind: Some("timeout".to_string()),
            dialog_trace: DialogTrace::default(),
        }
    }
}

/// Latency budget configuration v0.83.0
///
/// Explicit time budgets for razorback profile:
/// - Self-solve attempt: 500ms (no LLM needed)
/// - Junior reasoning: 5000ms
/// - Command pipeline: 3000ms
/// - Senior reasoning: 6000ms
/// - Total target: 15000ms
#[derive(Debug, Clone, Copy)]
pub struct LatencyBudget {
    /// Max self-solve attempt time (ms) - no LLM call needed
    pub max_self_solve_ms: u64,
    /// Max Junior LLM time (ms)
    pub max_junior_ms: u64,
    /// Max command pipeline execution time (ms)
    pub max_command_ms: u64,
    /// Max Senior LLM time (ms)
    pub max_senior_ms: u64,
    /// Max iterations for simple questions
    pub simple_question_max_iterations: u32,
}

impl Default for LatencyBudget {
    fn default() -> Self {
        Self {
            max_self_solve_ms: 500,
            max_junior_ms: 5000,
            max_command_ms: 3000,
            max_senior_ms: 6000,
            simple_question_max_iterations: 1,
        }
    }
}

impl LatencyBudget {
    /// Total budget for a question (target: 15s on razorback)
    pub fn total_budget_ms(&self) -> u64 {
        self.max_self_solve_ms + self.max_junior_ms + self.max_command_ms + self.max_senior_ms
    }

    /// Create a relaxed budget for complex questions
    pub fn complex() -> Self {
        Self {
            max_self_solve_ms: 500,
            max_junior_ms: 10000,
            max_command_ms: 5000,
            max_senior_ms: 10000,
            simple_question_max_iterations: 3,
        }
    }
}

/// Simple question classifier
/// A question is simple if it requires only one probe from the simple set
pub fn is_simple_question(probes: &[String]) -> bool {
    const SIMPLE_PROBES: &[&str] = &["cpu.info", "mem.info", "hardware.ram"];

    // Simple = exactly one probe, and it's in the simple set
    if probes.len() != 1 {
        return false;
    }

    SIMPLE_PROBES.contains(&probes[0].as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_answer_green() {
        let answer = StructuredAnswer::green(
            "You have 32 threads",
            vec!["16 physical cores".to_string(), "2 threads per core".to_string()],
            vec!["cpu.info: CPU(s): 32".to_string()],
        );
        assert!(answer.is_green());
        assert!(!answer.is_yellow());
        assert!(!answer.is_red());
        assert_eq!(answer.reliability_label, "Green");
    }

    #[test]
    fn test_structured_answer_timeout() {
        let answer = StructuredAnswer::timeout(15000, 18000);
        assert!(answer.is_red());
        assert!(answer.headline.contains("Timed out"));
    }

    #[test]
    fn test_is_simple_question() {
        assert!(is_simple_question(&["cpu.info".to_string()]));
        assert!(is_simple_question(&["mem.info".to_string()]));
        assert!(is_simple_question(&["hardware.ram".to_string()]));

        // Not simple: multiple probes
        assert!(!is_simple_question(&["cpu.info".to_string(), "mem.info".to_string()]));

        // Not simple: non-simple probe
        assert!(!is_simple_question(&["disk.lsblk".to_string()]));

        // Not simple: empty
        assert!(!is_simple_question(&[]));
    }

    #[test]
    fn test_latency_budget_default() {
        let budget = LatencyBudget::default();
        // v0.83.0: New time budgets for razorback
        assert_eq!(budget.max_self_solve_ms, 500);
        assert_eq!(budget.max_junior_ms, 5000);
        assert_eq!(budget.max_command_ms, 3000);
        assert_eq!(budget.max_senior_ms, 6000);
        assert_eq!(budget.simple_question_max_iterations, 1);
        // Total: 500 + 5000 + 3000 + 6000 = 14500ms
        assert_eq!(budget.total_budget_ms(), 14500);
    }

    #[test]
    fn test_latency_budget_complex() {
        let budget = LatencyBudget::complex();
        assert_eq!(budget.max_junior_ms, 10000);
        assert_eq!(budget.max_senior_ms, 10000);
        assert_eq!(budget.simple_question_max_iterations, 3);
    }

    #[test]
    fn test_dialog_trace() {
        let trace = DialogTrace::new(
            vec!["cpu.info".to_string()],
            true,
            "approve",
        );
        assert_eq!(trace.junior_plan_probes, vec!["cpu.info"]);
        assert!(trace.junior_had_draft);
        assert_eq!(trace.senior_verdict, "approve");
    }

    #[test]
    fn test_qa_output_serialization() {
        let qa = QaOutput::new(
            StructuredAnswer::green("Answer", vec![], vec![]),
            0.95,  // score_overall
            1200,
            800,
            1,
            vec!["cpu.info".to_string()],  // probes_used
            None,  // error_kind
            DialogTrace::new(vec!["cpu.info".to_string()], true, "approve"),
        );

        let json = serde_json::to_string(&qa).unwrap();
        assert!(json.contains("\"junior_ms\":1200"));
        assert!(json.contains("\"senior_ms\":800"));
        assert!(json.contains("\"iterations\":1"));
        assert!(json.contains("\"score_overall\":0.95"));
        assert!(json.contains("\"probes_used\":[\"cpu.info\"]"));
        // error_kind should not be serialized when None
        assert!(!json.contains("\"error_kind\""));
    }

    #[test]
    fn test_qa_output_error() {
        let qa = QaOutput::error("timeout", "Timed out", vec!["Budget exceeded".to_string()]);

        assert_eq!(qa.score_overall, 0.0);
        assert_eq!(qa.error_kind, Some("timeout".to_string()));
        assert!(qa.answer.is_red());

        let json = serde_json::to_string(&qa).unwrap();
        assert!(json.contains("\"error_kind\":\"timeout\""));
        assert!(json.contains("\"score_overall\":0.0"));
    }
}
