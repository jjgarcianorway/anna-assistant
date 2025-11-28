//! Structured Trace Pipeline v0.75.0
//!
//! Complete Debug Output Contract for Anna Assistant.
//! Every question produces a structured trace for debugging and auditing.
//!
//! Output locations:
//! - CLI: Human-readable debug block (5 sections)
//! - /var/log/anna/traces.jsonl: JSON traces (rotating)
//! - /var/log/anna/debug.log: Full debug with raw LLM messages (rotating)

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use tracing::{debug, warn};
use uuid::Uuid;

/// Log directory for traces
const TRACE_LOG_DIR: &str = "/var/log/anna";
const TRACE_LOG_FILE: &str = "traces.jsonl";
const DEBUG_LOG_FILE: &str = "debug.log";
const MAX_LOG_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

// ============================================================================
// v0.75.0 Debug Block - Complete 5-Section Contract
// ============================================================================

/// v0.75.0 Complete Debug Block with all 5 mandatory sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugBlock {
    /// Section 1: Input information
    pub input: InputSection,
    /// Section 2: Junior's plan
    pub junior_plan: JuniorPlanSection,
    /// Section 3: Probe execution details
    pub probes: ProbesSection,
    /// Section 4: Senior's verdict
    pub senior_verdict: SeniorVerdictSection,
    /// Section 5: Final answer
    pub final_answer: FinalAnswerSection,
    /// Raw LLM messages (for debug.log only, not CLI)
    pub raw_messages: RawLlmMessages,
}

/// Section 1: INPUT
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputSection {
    pub user_question: String,
    pub junior_model: String,
    pub senior_model: String,
    pub timestamp: String,
    pub iterations_used: usize,
}

/// Section 2: JUNIOR_PLAN
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JuniorPlanSection {
    pub intent: String,
    pub requested_probes: Vec<String>,
    pub raw_junior_plan: String,
    pub junior_reasoning: String,
    pub junior_score_plan_quality: f64,
}

/// Section 3: PROBES
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProbesSection {
    pub executed: Vec<ProbeExecution>,
    pub probe_failures: Vec<ProbeFailure>,
}

/// Individual probe execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeExecution {
    pub probe_id: String,
    pub status: String,
    pub duration_ms: u64,
    pub command: String,
    pub evidence_summary: String,
}

/// Probe failure record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeFailure {
    pub probe_id: String,
    pub error: String,
}

/// Section 4: SENIOR_VERDICT
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeniorVerdictSection {
    pub verdict: String,
    pub senior_analysis: String,
    pub suggested_fix: Option<String>,
    pub citations_validated: Vec<String>,
    pub scores: VerdictScores,
}

/// Score breakdown in Senior verdict
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VerdictScores {
    pub evidence: f64,
    pub reasoning: f64,
    pub coverage: f64,
    pub overall: f64,
}

/// Section 5: FINAL_ANSWER
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinalAnswerSection {
    pub text: String,
    pub reliability: String,
    pub confidence_score: f64,
}

/// Raw LLM messages for forensic analysis (debug.log only)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawLlmMessages {
    pub junior_prompt: String,
    pub junior_response: String,
    pub senior_prompt: String,
    pub senior_response: String,
}

impl DebugBlock {
    /// Create a new empty debug block
    pub fn new(question: &str, junior_model: &str, senior_model: &str) -> Self {
        Self {
            input: InputSection {
                user_question: question.to_string(),
                junior_model: junior_model.to_string(),
                senior_model: senior_model.to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                iterations_used: 0,
            },
            junior_plan: JuniorPlanSection::default(),
            probes: ProbesSection::default(),
            senior_verdict: SeniorVerdictSection::default(),
            final_answer: FinalAnswerSection::default(),
            raw_messages: RawLlmMessages::default(),
        }
    }

    /// Set Junior plan from LLM-A response
    pub fn set_junior_plan(
        &mut self,
        intent: &str,
        requested_probes: Vec<String>,
        raw_response: &str,
        reasoning: &str,
        plan_quality: f64,
    ) {
        self.junior_plan = JuniorPlanSection {
            intent: intent.to_string(),
            requested_probes,
            raw_junior_plan: raw_response.to_string(),
            junior_reasoning: reasoning.to_string(),
            junior_score_plan_quality: plan_quality,
        };
    }

    /// Add a successful probe execution
    pub fn add_probe_execution(
        &mut self,
        probe_id: &str,
        command: &str,
        duration_ms: u64,
        evidence: &str,
    ) {
        let summary = if evidence.len() > 200 {
            format!("{}...", &evidence[..200])
        } else if evidence.is_empty() {
            "<none>".to_string()
        } else {
            evidence.to_string()
        };

        self.probes.executed.push(ProbeExecution {
            probe_id: probe_id.to_string(),
            status: "ok".to_string(),
            duration_ms,
            command: command.to_string(),
            evidence_summary: summary,
        });
    }

    /// Add a failed probe execution
    pub fn add_probe_failure(&mut self, probe_id: &str, error: &str) {
        self.probes.probe_failures.push(ProbeFailure {
            probe_id: probe_id.to_string(),
            error: if error.is_empty() { "<none>".to_string() } else { error.to_string() },
        });

        // Also add to executed list with error status
        self.probes.executed.push(ProbeExecution {
            probe_id: probe_id.to_string(),
            status: "error".to_string(),
            duration_ms: 0,
            command: "<none>".to_string(),
            evidence_summary: error.to_string(),
        });
    }

    /// Set Senior verdict from LLM-B response
    pub fn set_senior_verdict(
        &mut self,
        verdict: &str,
        analysis: &str,
        suggested_fix: Option<String>,
        citations: Vec<String>,
        evidence: f64,
        reasoning: f64,
        coverage: f64,
    ) {
        let overall = evidence.min(reasoning).min(coverage);
        self.senior_verdict = SeniorVerdictSection {
            verdict: verdict.to_string(),
            senior_analysis: if analysis.is_empty() { "<none>".to_string() } else { analysis.to_string() },
            suggested_fix: suggested_fix.filter(|s| !s.is_empty()),
            citations_validated: citations,
            scores: VerdictScores {
                evidence,
                reasoning,
                coverage,
                overall,
            },
        };
    }

    /// Set final answer
    pub fn set_final_answer(&mut self, text: &str, confidence: f64) {
        let reliability = if confidence >= 0.90 {
            "GREEN"
        } else if confidence >= 0.70 {
            "YELLOW"
        } else {
            "RED"
        };

        self.final_answer = FinalAnswerSection {
            text: if text.is_empty() { "<none>".to_string() } else { text.to_string() },
            reliability: reliability.to_string(),
            confidence_score: confidence,
        };
    }

    /// Set raw LLM messages for forensic logging
    pub fn set_raw_messages(
        &mut self,
        junior_prompt: &str,
        junior_response: &str,
        senior_prompt: &str,
        senior_response: &str,
    ) {
        self.raw_messages = RawLlmMessages {
            junior_prompt: junior_prompt.to_string(),
            junior_response: junior_response.to_string(),
            senior_prompt: senior_prompt.to_string(),
            senior_response: senior_response.to_string(),
        };
    }

    /// Format as human-readable CLI debug output (5 sections, ASCII-only)
    pub fn format_cli(&self) -> String {
        let mut out = String::new();

        // Header
        out.push_str("\n");
        out.push_str("+==============================================================================+\n");
        out.push_str("|                           ANNA DEBUG OUTPUT v0.75.0                         |\n");
        out.push_str("+==============================================================================+\n");

        // Section 1: INPUT
        out.push_str("\n[INPUT]\n");
        out.push_str(&format!("user_question: \"{}\"\n", self.input.user_question));
        out.push_str(&format!("junior_model: \"{}\"\n", self.input.junior_model));
        out.push_str(&format!("senior_model: \"{}\"\n", self.input.senior_model));
        out.push_str(&format!("timestamp: \"{}\"\n", self.input.timestamp));
        out.push_str(&format!("iterations_used: {}\n", self.input.iterations_used));

        // Section 2: JUNIOR_PLAN
        out.push_str("\n[JUNIOR_PLAN]\n");
        let intent = if self.junior_plan.intent.is_empty() { "<none>" } else { &self.junior_plan.intent };
        out.push_str(&format!("intent: \"{}\"\n", intent));
        if self.junior_plan.requested_probes.is_empty() {
            out.push_str("requested_probes: []\n");
        } else {
            out.push_str(&format!("requested_probes: {:?}\n", self.junior_plan.requested_probes));
        }
        let raw_plan = if self.junior_plan.raw_junior_plan.is_empty() {
            "<none>".to_string()
        } else {
            // Truncate for CLI
            let truncated: String = self.junior_plan.raw_junior_plan.chars().take(500).collect();
            if self.junior_plan.raw_junior_plan.len() > 500 {
                format!("{}...", truncated)
            } else {
                truncated
            }
        };
        out.push_str(&format!("raw_junior_plan: \"{}\"\n", raw_plan.replace('\n', " ")));
        let reasoning = if self.junior_plan.junior_reasoning.is_empty() { "<none>" } else { &self.junior_plan.junior_reasoning };
        out.push_str(&format!("junior_reasoning: \"{}\"\n", reasoning));
        out.push_str(&format!("junior_score_plan_quality: {:.2}\n", self.junior_plan.junior_score_plan_quality));

        // Section 3: PROBES
        out.push_str("\n[PROBES]\n");
        out.push_str("executed: [\n");
        if self.probes.executed.is_empty() {
            out.push_str("  (none)\n");
        } else {
            for probe in &self.probes.executed {
                out.push_str("  {\n");
                out.push_str(&format!("    probe_id: \"{}\",\n", probe.probe_id));
                out.push_str(&format!("    status: \"{}\",\n", probe.status));
                out.push_str(&format!("    duration_ms: {},\n", probe.duration_ms));
                out.push_str(&format!("    command: \"{}\",\n", probe.command));
                let summary = probe.evidence_summary.replace('\n', " ");
                out.push_str(&format!("    evidence_summary: \"{}\"\n", summary));
                out.push_str("  },\n");
            }
        }
        out.push_str("]\n");
        out.push_str("probe_failures: [\n");
        if self.probes.probe_failures.is_empty() {
            out.push_str("  (none)\n");
        } else {
            for failure in &self.probes.probe_failures {
                out.push_str("  {\n");
                out.push_str(&format!("    probe_id: \"{}\",\n", failure.probe_id));
                out.push_str(&format!("    error: \"{}\"\n", failure.error));
                out.push_str("  },\n");
            }
        }
        out.push_str("]\n");

        // Section 4: SENIOR_VERDICT
        out.push_str("\n[SENIOR_VERDICT]\n");
        let verdict = if self.senior_verdict.verdict.is_empty() { "<none>" } else { &self.senior_verdict.verdict };
        out.push_str(&format!("verdict: \"{}\"\n", verdict));
        out.push_str(&format!("senior_analysis: \"{}\"\n", self.senior_verdict.senior_analysis));
        let fix = self.senior_verdict.suggested_fix.as_deref().unwrap_or("<none>");
        out.push_str(&format!("suggested_fix: \"{}\"\n", fix));
        if self.senior_verdict.citations_validated.is_empty() {
            out.push_str("citations_validated: []\n");
        } else {
            out.push_str(&format!("citations_validated: {:?}\n", self.senior_verdict.citations_validated));
        }
        out.push_str("scores: {\n");
        out.push_str(&format!("  evidence: {:.2},\n", self.senior_verdict.scores.evidence));
        out.push_str(&format!("  reasoning: {:.2},\n", self.senior_verdict.scores.reasoning));
        out.push_str(&format!("  coverage: {:.2},\n", self.senior_verdict.scores.coverage));
        out.push_str(&format!("  overall: {:.2}\n", self.senior_verdict.scores.overall));
        out.push_str("}\n");

        // Section 5: FINAL_ANSWER
        out.push_str("\n[FINAL_ANSWER]\n");
        let text = if self.final_answer.text.len() > 300 {
            format!("{}...", &self.final_answer.text[..300])
        } else {
            self.final_answer.text.clone()
        };
        out.push_str(&format!("text: \"{}\"\n", text.replace('\n', " ")));
        out.push_str(&format!("reliability: \"{}\"\n", self.final_answer.reliability));
        out.push_str(&format!("confidence_score: {:.2}\n", self.final_answer.confidence_score));

        out.push_str("\n+==============================================================================+\n");
        out
    }

    /// Write to /var/log/anna/debug.log with raw LLM messages
    pub fn write_debug_log(&self) -> std::io::Result<()> {
        let log_dir = Path::new(TRACE_LOG_DIR);
        if !log_dir.exists() {
            if let Err(e) = fs::create_dir_all(log_dir) {
                warn!("Could not create log directory: {}", e);
                return Ok(()); // Non-fatal
            }
        }

        let log_path = log_dir.join(DEBUG_LOG_FILE);

        // Rotate if file too large
        if log_path.exists() {
            if let Ok(meta) = fs::metadata(&log_path) {
                if meta.len() > MAX_LOG_FILE_SIZE {
                    let rotated = log_dir.join(format!("debug.{}.log", chrono::Utc::now().format("%Y%m%d%H%M%S")));
                    let _ = fs::rename(&log_path, rotated);
                }
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        // Write CLI output first
        writeln!(file, "{}", self.format_cli())?;

        // Write raw LLM messages
        writeln!(file, "\n[RAW_JUNIOR_MESSAGE]")?;
        writeln!(file, "=== PROMPT ===")?;
        writeln!(file, "{}", if self.raw_messages.junior_prompt.is_empty() { "<none>" } else { &self.raw_messages.junior_prompt })?;
        writeln!(file, "=== RESPONSE ===")?;
        writeln!(file, "{}", if self.raw_messages.junior_response.is_empty() { "<none>" } else { &self.raw_messages.junior_response })?;

        writeln!(file, "\n[RAW_SENIOR_MESSAGE]")?;
        writeln!(file, "=== PROMPT ===")?;
        writeln!(file, "{}", if self.raw_messages.senior_prompt.is_empty() { "<none>" } else { &self.raw_messages.senior_prompt })?;
        writeln!(file, "=== RESPONSE ===")?;
        writeln!(file, "{}", if self.raw_messages.senior_response.is_empty() { "<none>" } else { &self.raw_messages.senior_response })?;

        writeln!(file, "\n{}", "=".repeat(80))?;

        debug!("Debug log written: {}", self.input.timestamp);
        Ok(())
    }

    /// Write to /var/log/anna/traces.jsonl
    pub fn write_json_trace(&self) -> std::io::Result<()> {
        let log_dir = Path::new(TRACE_LOG_DIR);
        if !log_dir.exists() {
            if let Err(e) = fs::create_dir_all(log_dir) {
                warn!("Could not create log directory: {}", e);
                return Ok(()); // Non-fatal
            }
        }

        let log_path = log_dir.join(TRACE_LOG_FILE);

        // Rotate if file too large
        if log_path.exists() {
            if let Ok(meta) = fs::metadata(&log_path) {
                if meta.len() > MAX_LOG_FILE_SIZE {
                    let rotated = log_dir.join(format!("traces.{}.jsonl", chrono::Utc::now().format("%Y%m%d%H%M%S")));
                    let _ = fs::rename(&log_path, rotated);
                }
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        let json = serde_json::to_string(self).unwrap_or_default();
        writeln!(file, "{}", json)?;

        debug!("JSON trace written: {}", self.input.timestamp);
        Ok(())
    }

    /// Write both logs
    pub fn write_all_logs(&self) -> std::io::Result<()> {
        self.write_json_trace()?;
        self.write_debug_log()?;
        Ok(())
    }
}

// ============================================================================
// Legacy v0.74.0 Structures (kept for backwards compatibility)
// ============================================================================

/// Structured trace for a single question/answer cycle (v0.74.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionTrace {
    pub timestamp: String,
    pub correlation_id: String,
    pub question: String,
    pub iterations: usize,
    pub junior_plan: JuniorPlan,
    pub probes_run: Vec<ProbeTrace>,
    pub senior_verdict: SeniorVerdict,
    pub final_answer_summary: String,
    pub duration_ms: u64,
}

/// Junior's plan from LLM-A (v0.74.0)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JuniorPlan {
    pub intent: String,
    #[serde(default)]
    pub substeps: Vec<String>,
    #[serde(default)]
    pub requested_probes: Vec<String>,
    #[serde(default)]
    pub requested_commands: Vec<String>,
}

/// Trace of a single probe execution (v0.74.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeTrace {
    pub id: String,
    pub command: String,
    pub status: String,
    pub duration_ms: u64,
}

/// Senior's verdict from LLM-B (v0.74.0)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeniorVerdict {
    pub evidence: f64,
    pub reasoning: f64,
    pub coverage: f64,
    pub overall: f64,
    pub color: String,
    #[serde(default)]
    pub problems: Vec<String>,
}

impl QuestionTrace {
    pub fn new(question: &str) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            correlation_id: generate_correlation_id(),
            question: question.to_string(),
            iterations: 0,
            junior_plan: JuniorPlan::default(),
            probes_run: vec![],
            senior_verdict: SeniorVerdict::default(),
            final_answer_summary: String::new(),
            duration_ms: 0,
        }
    }

    pub fn add_probe(&mut self, id: &str, command: &str, status: &str, duration_ms: u64) {
        self.probes_run.push(ProbeTrace {
            id: id.to_string(),
            command: command.to_string(),
            status: status.to_string(),
            duration_ms,
        });
    }

    pub fn set_verdict(&mut self, evidence: f64, reasoning: f64, coverage: f64, problems: Vec<String>) {
        let overall = evidence.min(reasoning).min(coverage);
        let color = if overall >= 0.90 { "green" } else if overall >= 0.70 { "yellow" } else { "red" };
        self.senior_verdict = SeniorVerdict {
            evidence,
            reasoning,
            coverage,
            overall,
            color: color.to_string(),
            problems,
        };
    }

    pub fn write_to_log(&self) -> std::io::Result<()> {
        let log_dir = Path::new(TRACE_LOG_DIR);
        if !log_dir.exists() {
            if let Err(e) = fs::create_dir_all(log_dir) {
                warn!("Could not create trace log directory: {}", e);
                return Ok(());
            }
        }

        let log_path = log_dir.join(TRACE_LOG_FILE);
        if log_path.exists() {
            if let Ok(meta) = fs::metadata(&log_path) {
                if meta.len() > MAX_LOG_FILE_SIZE {
                    let rotated = log_dir.join(format!("traces.{}.jsonl", chrono::Utc::now().format("%Y%m%d%H%M%S")));
                    let _ = fs::rename(&log_path, rotated);
                }
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        let json = serde_json::to_string(self).unwrap_or_default();
        writeln!(file, "{}", json)?;

        debug!("Trace written: {}", self.correlation_id);
        Ok(())
    }

    pub fn format_debug(&self) -> String {
        let mut output = String::new();
        output.push_str("\nDEBUG TRACE\n");
        output.push_str("------------------------------------------------------------\n");
        let probes = if self.junior_plan.requested_probes.is_empty() {
            "none".to_string()
        } else {
            self.junior_plan.requested_probes.join(" -> ")
        };
        output.push_str(&format!("Plan: {}\n", probes));
        output.push_str("Probes:\n");
        if self.probes_run.is_empty() {
            output.push_str("  (none)\n");
        } else {
            for probe in &self.probes_run {
                output.push_str(&format!("  - {}: {} ({} ms)\n", probe.id, probe.status, probe.duration_ms));
            }
        }
        let color = self.senior_verdict.color.to_uppercase();
        output.push_str(&format!("Senior verdict: {} (overall {:.2})\n", color, self.senior_verdict.overall));
        output.push_str(&format!("Iterations: {}\n", self.iterations));
        output.push_str("------------------------------------------------------------\n");
        output
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Generate a unique correlation ID
pub fn generate_correlation_id() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}

/// Check if debug mode is enabled via environment variable
pub fn is_debug_mode() -> bool {
    std::env::var("ANNA_DEBUG").is_ok()
}

/// Check if debug is enabled via config file
pub fn is_debug_enabled_by_config() -> bool {
    // TODO: Read from /etc/anna/config.toml
    // For now, just check environment variable
    is_debug_mode()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_generation() {
        let id1 = generate_correlation_id();
        let id2 = generate_correlation_id();
        assert_eq!(id1.len(), 8);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_trace_creation() {
        let trace = QuestionTrace::new("What CPU do I have?");
        assert!(!trace.correlation_id.is_empty());
        assert_eq!(trace.question, "What CPU do I have?");
        assert_eq!(trace.iterations, 0);
    }

    #[test]
    fn test_add_probe() {
        let mut trace = QuestionTrace::new("test");
        trace.add_probe("cpu.info", "lscpu -J", "ok", 5);
        assert_eq!(trace.probes_run.len(), 1);
        assert_eq!(trace.probes_run[0].id, "cpu.info");
    }

    #[test]
    fn test_set_verdict() {
        let mut trace = QuestionTrace::new("test");
        trace.set_verdict(0.95, 0.90, 0.92, vec![]);
        assert_eq!(trace.senior_verdict.overall, 0.90);
        assert_eq!(trace.senior_verdict.color, "green");
    }

    #[test]
    fn test_verdict_colors() {
        let mut trace = QuestionTrace::new("test");
        trace.set_verdict(0.95, 0.95, 0.95, vec![]);
        assert_eq!(trace.senior_verdict.color, "green");
        trace.set_verdict(0.80, 0.80, 0.80, vec![]);
        assert_eq!(trace.senior_verdict.color, "yellow");
        trace.set_verdict(0.50, 0.50, 0.50, vec![]);
        assert_eq!(trace.senior_verdict.color, "red");
    }

    #[test]
    fn test_debug_format() {
        let mut trace = QuestionTrace::new("What CPU do I have?");
        trace.junior_plan.requested_probes = vec!["cpu.info".to_string()];
        trace.add_probe("cpu.info", "lscpu -J", "ok", 3);
        trace.set_verdict(0.95, 0.90, 0.92, vec![]);
        trace.iterations = 1;
        let debug_output = trace.format_debug();
        assert!(debug_output.contains("DEBUG TRACE"));
        assert!(debug_output.contains("cpu.info"));
        assert!(debug_output.contains("GREEN"));
        assert!(debug_output.contains("Iterations: 1"));
    }

    #[test]
    fn test_trace_serialization() {
        let mut trace = QuestionTrace::new("test");
        trace.add_probe("cpu.info", "lscpu -J", "ok", 5);
        trace.set_verdict(0.95, 0.90, 0.85, vec![]);
        let json = serde_json::to_string(&trace).unwrap();
        assert!(json.contains("correlation_id"));
        assert!(json.contains("cpu.info"));
        assert!(json.contains("0.85"));
    }

    // v0.75.0 DebugBlock tests
    #[test]
    fn test_debug_block_creation() {
        let block = DebugBlock::new("What CPU?", "qwen2.5:7b", "qwen2.5:14b");
        assert_eq!(block.input.user_question, "What CPU?");
        assert_eq!(block.input.junior_model, "qwen2.5:7b");
        assert_eq!(block.input.senior_model, "qwen2.5:14b");
    }

    #[test]
    fn test_debug_block_junior_plan() {
        let mut block = DebugBlock::new("test", "junior", "senior");
        block.set_junior_plan(
            "hardware_info",
            vec!["cpu.info".to_string()],
            "raw response here",
            "I will query CPU info",
            0.85,
        );
        assert_eq!(block.junior_plan.intent, "hardware_info");
        assert_eq!(block.junior_plan.requested_probes, vec!["cpu.info"]);
        assert_eq!(block.junior_plan.junior_score_plan_quality, 0.85);
    }

    #[test]
    fn test_debug_block_probes() {
        let mut block = DebugBlock::new("test", "junior", "senior");
        block.add_probe_execution("cpu.info", "lscpu -J", 50, "CPU: Intel i9");
        block.add_probe_failure("missing.probe", "probe not implemented");
        assert_eq!(block.probes.executed.len(), 2); // success + failure added to executed
        assert_eq!(block.probes.probe_failures.len(), 1);
    }

    #[test]
    fn test_debug_block_senior_verdict() {
        let mut block = DebugBlock::new("test", "junior", "senior");
        block.set_senior_verdict(
            "approve",
            "Answer is complete",
            None,
            vec!["cpu.info".to_string()],
            0.95,
            0.90,
            0.92,
        );
        assert_eq!(block.senior_verdict.verdict, "approve");
        assert_eq!(block.senior_verdict.scores.overall, 0.90); // min
    }

    #[test]
    fn test_debug_block_final_answer() {
        let mut block = DebugBlock::new("test", "junior", "senior");
        block.set_final_answer("Your CPU is Intel i9", 0.95);
        assert_eq!(block.final_answer.text, "Your CPU is Intel i9");
        assert_eq!(block.final_answer.reliability, "GREEN");
    }

    #[test]
    fn test_debug_block_cli_format() {
        let mut block = DebugBlock::new("What CPU?", "junior", "senior");
        block.input.iterations_used = 1;
        block.set_junior_plan(
            "hardware_info",
            vec!["cpu.info".to_string()],
            "raw",
            "reasoning",
            0.85,
        );
        block.add_probe_execution("cpu.info", "lscpu", 50, "Intel i9");
        block.set_senior_verdict(
            "approve",
            "Looks good",
            None,
            vec!["cpu.info".to_string()],
            0.95,
            0.90,
            0.92,
        );
        block.set_final_answer("Your CPU is Intel i9", 0.90);

        let output = block.format_cli();
        assert!(output.contains("[INPUT]"));
        assert!(output.contains("[JUNIOR_PLAN]"));
        assert!(output.contains("[PROBES]"));
        assert!(output.contains("[SENIOR_VERDICT]"));
        assert!(output.contains("[FINAL_ANSWER]"));
        assert!(output.contains("hardware_info"));
        assert!(output.contains("cpu.info"));
        assert!(output.contains("approve"));
        assert!(output.contains("GREEN"));
    }

    #[test]
    fn test_debug_block_none_values() {
        let block = DebugBlock::new("", "", "");
        let output = block.format_cli();
        // Should handle empty values with <none>
        assert!(output.contains("[INPUT]"));
        assert!(output.contains("[JUNIOR_PLAN]"));
    }

    #[test]
    fn test_debug_block_serialization() {
        let mut block = DebugBlock::new("test", "junior", "senior");
        block.set_final_answer("answer", 0.90);
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("user_question"));
        assert!(json.contains("junior_model"));
        assert!(json.contains("confidence_score"));
    }
}
