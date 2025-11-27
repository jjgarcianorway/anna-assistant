//! Research Trace Logging v0.15.0
//!
//! JSONL logging for the research loop, capturing:
//! - Each LLM-A iteration
//! - Check requests and approvals
//! - Command executions
//! - LLM-B evaluations
//! - User questions and answers
//! - Final answer with confidence

use crate::answer_engine::protocol_v15::{
    CheckResult, LlmAResponseV15, LlmBResponseV15, LlmBVerdict, ReasoningTrace, TraceStep,
    UserAnswer, UserQuestion,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// Default trace log directory
pub const TRACE_LOG_DIR: &str = "/var/log/anna/traces";

/// A single trace entry in JSONL format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTraceEntry {
    /// Unique trace ID (UUID)
    pub trace_id: String,
    /// Entry sequence number within trace
    pub sequence: usize,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Entry type
    #[serde(flatten)]
    pub entry: TraceEntryType,
}

/// Types of trace entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TraceEntryType {
    /// Research loop started
    Start { user_request: String, mode: String },
    /// LLM-A iteration
    LlmAIteration {
        iteration: usize,
        intent: String,
        plan_steps: Vec<String>,
        check_count: usize,
        has_question: bool,
        has_draft: bool,
        needs_mentor: bool,
        self_confidence: f64,
    },
    /// Check requested
    CheckRequested {
        check_id: String,
        command: String,
        risk: String,
        reason: String,
    },
    /// Check approved/denied
    CheckApproval {
        check_id: String,
        approved: bool,
        denial_reason: Option<String>,
    },
    /// Check executed
    CheckExecuted {
        check_id: String,
        exit_code: i32,
        output_preview: Option<String>,
        duration_ms: u64,
    },
    /// LLM-B evaluation
    LlmBEvaluation {
        verdict: String,
        confidence: f64,
        problems_count: usize,
        mentor_feedback: Option<String>,
        mentor_score: Option<f64>,
    },
    /// User question asked
    UserQuestion {
        question: String,
        style: String,
        options_count: usize,
    },
    /// User answer received
    UserAnswer { answer_summary: String },
    /// Research loop complete
    Complete {
        final_answer_preview: String,
        confidence: f64,
        total_iterations: usize,
        total_checks: usize,
        duration_ms: u64,
    },
    /// Research loop failed
    Failed { reason: String, iteration: usize },
}

/// Research trace logger
pub struct ResearchTraceLogger {
    /// Trace ID
    trace_id: String,
    /// Current sequence number
    sequence: usize,
    /// Start time for duration calculation
    start_time: DateTime<Utc>,
    /// Log file path
    log_path: PathBuf,
    /// Buffered writer (optional for batch writes)
    entries: Vec<ResearchTraceEntry>,
}

impl ResearchTraceLogger {
    /// Create a new trace logger
    pub fn new(trace_id: &str) -> Self {
        let log_dir = PathBuf::from(TRACE_LOG_DIR);
        let date = Utc::now().format("%Y-%m-%d");
        let log_path = log_dir.join(format!("research-{}.jsonl", date));

        Self {
            trace_id: trace_id.to_string(),
            sequence: 0,
            start_time: Utc::now(),
            log_path,
            entries: Vec::new(),
        }
    }

    /// Log the start of a research loop
    pub fn log_start(&mut self, user_request: &str, mode: &str) {
        self.add_entry(TraceEntryType::Start {
            user_request: Self::truncate(user_request, 200),
            mode: mode.to_string(),
        });
    }

    /// Log an LLM-A iteration
    pub fn log_llm_a(&mut self, iteration: usize, response: &LlmAResponseV15) {
        self.add_entry(TraceEntryType::LlmAIteration {
            iteration,
            intent: response.intent.clone(),
            plan_steps: response.plan_steps.clone(),
            check_count: response.check_requests.len(),
            has_question: response.user_question.is_some(),
            has_draft: response.draft_answer.is_some(),
            needs_mentor: response.needs_mentor,
            self_confidence: response.self_confidence,
        });
    }

    /// Log a check request
    pub fn log_check_requested(&mut self, check_id: &str, command: &str, risk: &str, reason: &str) {
        self.add_entry(TraceEntryType::CheckRequested {
            check_id: check_id.to_string(),
            command: Self::truncate(command, 200),
            risk: risk.to_string(),
            reason: Self::truncate(reason, 100),
        });
    }

    /// Log check approval decision
    pub fn log_check_approval(
        &mut self,
        check_id: &str,
        approved: bool,
        denial_reason: Option<&str>,
    ) {
        self.add_entry(TraceEntryType::CheckApproval {
            check_id: check_id.to_string(),
            approved,
            denial_reason: denial_reason.map(|s| s.to_string()),
        });
    }

    /// Log check execution result
    pub fn log_check_executed(&mut self, result: &CheckResult, duration_ms: u64) {
        self.add_entry(TraceEntryType::CheckExecuted {
            check_id: result.check_id.clone(),
            exit_code: result.exit_code,
            output_preview: result.stdout.as_ref().map(|s| Self::truncate(s, 100)),
            duration_ms,
        });
    }

    /// Log LLM-B evaluation
    pub fn log_llm_b(&mut self, response: &LlmBResponseV15) {
        self.add_entry(TraceEntryType::LlmBEvaluation {
            verdict: response.verdict.as_str().to_string(),
            confidence: response.confidence,
            problems_count: response.problems.len(),
            mentor_feedback: response.mentor_feedback.clone(),
            mentor_score: response.mentor_score,
        });
    }

    /// Log user question
    pub fn log_user_question(&mut self, question: &UserQuestion) {
        self.add_entry(TraceEntryType::UserQuestion {
            question: Self::truncate(&question.question, 200),
            style: format!("{:?}", question.style),
            options_count: question.options.len(),
        });
    }

    /// Log user answer
    pub fn log_user_answer(&mut self, answer: &UserAnswer) {
        let summary = match &answer.answer {
            crate::UserAnswerValue::Single(s) => s.clone(),
            crate::UserAnswerValue::Multiple(v) => v.join(", "),
            crate::UserAnswerValue::Text(t) => Self::truncate(t, 100),
        };
        self.add_entry(TraceEntryType::UserAnswer {
            answer_summary: summary,
        });
    }

    /// Log successful completion
    pub fn log_complete(
        &mut self,
        answer: &str,
        confidence: f64,
        iterations: usize,
        checks: usize,
    ) {
        let duration_ms = (Utc::now() - self.start_time).num_milliseconds() as u64;
        self.add_entry(TraceEntryType::Complete {
            final_answer_preview: Self::truncate(answer, 200),
            confidence,
            total_iterations: iterations,
            total_checks: checks,
            duration_ms,
        });
        self.flush();
    }

    /// Log failure
    pub fn log_failed(&mut self, reason: &str, iteration: usize) {
        self.add_entry(TraceEntryType::Failed {
            reason: reason.to_string(),
            iteration,
        });
        self.flush();
    }

    /// Add an entry to the buffer
    fn add_entry(&mut self, entry: TraceEntryType) {
        self.sequence += 1;
        self.entries.push(ResearchTraceEntry {
            trace_id: self.trace_id.clone(),
            sequence: self.sequence,
            timestamp: Utc::now(),
            entry,
        });
    }

    /// Flush entries to disk
    pub fn flush(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        // Ensure directory exists
        if let Some(parent) = self.log_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Append to log file
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let mut writer = BufWriter::new(file);
            for entry in &self.entries {
                if let Ok(json) = serde_json::to_string(entry) {
                    let _ = writeln!(writer, "{}", json);
                }
            }
            let _ = writer.flush();
        }

        self.entries.clear();
    }

    /// Truncate string to max length
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }
}

impl Drop for ResearchTraceLogger {
    fn drop(&mut self) {
        // Ensure any remaining entries are flushed
        self.flush();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_trace_entry_serialization() {
        let entry = ResearchTraceEntry {
            trace_id: "test-123".to_string(),
            sequence: 1,
            timestamp: Utc::now(),
            entry: TraceEntryType::Start {
                user_request: "How much RAM?".to_string(),
                mode: "normal".to_string(),
            },
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"type\":\"start\""));
        assert!(json.contains("\"user_request\":\"How much RAM?\""));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(ResearchTraceLogger::truncate("short", 100), "short");
        assert_eq!(ResearchTraceLogger::truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_logger_creation() {
        let logger = ResearchTraceLogger::new("test-trace-001");
        assert_eq!(logger.trace_id(), "test-trace-001");
        assert_eq!(logger.sequence, 0);
    }

    #[test]
    fn test_log_start() {
        let mut logger = ResearchTraceLogger::new("test-001");
        logger.log_start("How many CPU cores?", "normal");
        assert_eq!(logger.entries.len(), 1);
        assert_eq!(logger.sequence, 1);
    }

    #[test]
    fn test_log_complete() {
        let mut logger = ResearchTraceLogger::new("test-002");
        // Create a temp dir for the log file
        logger.log_path = std::env::temp_dir().join("test-trace.jsonl");
        logger.log_start("Test question", "dev");
        logger.log_complete("The answer is 42", 0.95, 2, 3);
        // Entries should be flushed
        assert!(logger.entries.is_empty());
    }

    #[test]
    fn test_llm_b_evaluation_serialization() {
        let entry = TraceEntryType::LlmBEvaluation {
            verdict: "accept".to_string(),
            confidence: 0.92,
            problems_count: 0,
            mentor_feedback: None,
            mentor_score: Some(0.85),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"type\":\"llm_b_evaluation\""));
        assert!(json.contains("\"verdict\":\"accept\""));
        assert!(json.contains("\"mentor_score\":0.85"));
    }
}
