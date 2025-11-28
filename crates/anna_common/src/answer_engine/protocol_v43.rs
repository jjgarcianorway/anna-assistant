//! Protocol v0.72.0 - Live Debug Streaming
//!
//! Real-time streaming debug events for the Junior/Senior orchestration loop.
//! Events are emitted as they happen, not buffered until the end.
//!
//! ## Event Types
//!
//! - ITERATION_STARTED: New iteration beginning
//! - JUNIOR_PLAN_STARTED: Junior LLM starting to plan
//! - JUNIOR_PLAN_DONE: Junior LLM finished planning (with intent summary)
//! - ANNA_PROBE: Anna executing a probe/command
//! - PROBES_EXECUTED: All probes executed and results collected
//! - SENIOR_REVIEW_STARTED: Senior LLM starting review
//! - SENIOR_REVIEW_DONE: Senior LLM finished with verdict (with percentage)
//! - RETRY_STARTED: Starting retry iteration
//! - ANSWER_READY: Final answer synthesized
//!
//! ## Wire Format
//!
//! Events are sent as newline-delimited JSON (NDJSON).
//! Each line is a complete JSON object that can be parsed independently.
//!
//! ## Activation
//!
//! Live debug streaming is activated by:
//! - Environment variable: ANNA_DEBUG=1
//! - Config flag: debug.live_view = true

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Protocol version for v0.72.0
pub const PROTOCOL_VERSION_V43: &str = "0.72.0";

/// Debug event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DebugEventType {
    /// New iteration starting
    IterationStarted,
    /// Junior LLM starting to produce plan
    JuniorPlanStarted,
    /// Junior LLM finished planning
    JuniorPlanDone,
    /// Anna executing a single probe/command
    AnnaProbe,
    /// Probes executed, evidence collected
    ProbesExecuted,
    /// Senior LLM starting review
    SeniorReviewStarted,
    /// Senior LLM finished with verdict
    SeniorReviewDone,
    /// Starting a retry iteration
    RetryStarted,
    /// Final answer is ready
    AnswerReady,
    /// Error occurred during processing
    Error,
    /// Stream starting (initial handshake)
    StreamStarted,
    /// Stream ending (final event)
    StreamEnded,
    /// v0.76.2: LLM prompt being sent (full dialog view)
    LlmPromptSent,
    /// v0.76.2: LLM response received (full dialog view)
    LlmResponseReceived,
}

impl DebugEventType {
    /// Get human-readable label for display (v0.72.0 conversational voice)
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::IterationStarted => "ITERATION",
            Self::JuniorPlanStarted => "JUNIOR: PLAN",
            Self::JuniorPlanDone => "JUNIOR: PLAN",
            Self::AnnaProbe => "ANNA: PROBE",
            Self::ProbesExecuted => "ANNA: PROBES",
            Self::SeniorReviewStarted => "SENIOR: REVIEW",
            Self::SeniorReviewDone => "SENIOR: VERDICT",
            Self::RetryStarted => "RETRY",
            Self::AnswerReady => "ANNA: DONE",
            Self::Error => "ERROR",
            Self::StreamStarted => "STREAM: START",
            Self::StreamEnded => "STREAM: END",
            Self::LlmPromptSent => "LLM: PROMPT",
            Self::LlmResponseReceived => "LLM: RESPONSE",
        }
    }

    /// Get ANSI color code for this event type (true color)
    pub fn color_code(&self) -> &'static str {
        match self {
            Self::IterationStarted => "\x1b[38;2;100;149;237m",  // Cornflower blue
            Self::JuniorPlanStarted => "\x1b[38;2;255;165;0m",   // Orange
            Self::JuniorPlanDone => "\x1b[38;2;50;205;50m",      // Lime green
            Self::AnnaProbe => "\x1b[38;2;0;191;255m",           // Deep sky blue
            Self::ProbesExecuted => "\x1b[38;2;138;43;226m",     // Blue violet
            Self::SeniorReviewStarted => "\x1b[38;2;255;215;0m", // Gold
            Self::SeniorReviewDone => "\x1b[38;2;0;255;127m",    // Spring green
            Self::RetryStarted => "\x1b[38;2;255;99;71m",        // Tomato
            Self::AnswerReady => "\x1b[38;2;0;255;0m",           // Green
            Self::Error => "\x1b[38;2;255;0;0m",                 // Red
            Self::StreamStarted => "\x1b[38;2;169;169;169m",     // Dark gray
            Self::StreamEnded => "\x1b[38;2;169;169;169m",       // Dark gray
            Self::LlmPromptSent => "\x1b[38;2;0;255;255m",       // Cyan (prompt going out)
            Self::LlmResponseReceived => "\x1b[38;2;255;0;255m", // Magenta (response coming in)
        }
    }
}

/// A single debug event for real-time streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEvent {
    /// Event type
    pub event_type: DebugEventType,
    /// ISO8601 timestamp
    pub timestamp: DateTime<Utc>,
    /// Iteration number (1-based, 0 for non-iteration events)
    pub iteration: usize,
    /// Brief human-readable description
    pub description: String,
    /// Optional data snippets (truncated for display)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<DebugEventData>,
    /// Duration in milliseconds since last event
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
}

impl DebugEvent {
    /// Create a new debug event
    pub fn new(event_type: DebugEventType, iteration: usize, description: impl Into<String>) -> Self {
        Self {
            event_type,
            timestamp: Utc::now(),
            iteration,
            description: description.into(),
            data: None,
            elapsed_ms: None,
        }
    }

    /// Add data to the event
    pub fn with_data(mut self, data: DebugEventData) -> Self {
        self.data = Some(data);
        self
    }

    /// Add elapsed time
    pub fn with_elapsed(mut self, elapsed_ms: u64) -> Self {
        self.elapsed_ms = Some(elapsed_ms);
        self
    }

    /// Serialize to NDJSON line (newline-delimited JSON)
    pub fn to_ndjson(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format for terminal display (ASCII only, no emojis)
    pub fn format_terminal(&self) -> String {
        let reset = "\x1b[0m";
        let bold = "\x1b[1m";
        let dim = "\x1b[2m";
        let color = self.event_type.color_code();

        let mut output = String::new();

        // Timestamp and event type header
        let ts = self.timestamp.format("%H:%M:%S%.3f");
        output.push_str(&format!(
            "{dim}[{ts}]{reset} {bold}{color}[{label}]{reset}",
            label = self.event_type.as_label()
        ));

        // Iteration number if applicable
        if self.iteration > 0 {
            output.push_str(&format!(" {dim}(iter {}){reset}", self.iteration));
        }

        // Elapsed time if present
        if let Some(ms) = self.elapsed_ms {
            output.push_str(&format!(" {dim}+{ms}ms{reset}"));
        }

        output.push('\n');

        // Description
        if !self.description.is_empty() {
            output.push_str(&format!("  {}\n", self.description));
        }

        // Data snippets (indented)
        if let Some(ref data) = self.data {
            output.push_str(&data.format_indented(2));
        }

        output
    }
}

/// Data snippets attached to debug events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DebugEventData {
    /// Junior plan data
    JuniorPlan {
        intent: String,
        probes_requested: Vec<String>,
        has_draft: bool,
    },
    /// Single probe execution (v0.72.0)
    AnnaProbe {
        probe_id: String,
        command: String,
        latency_ms: u64,
        success: bool,
    },
    /// Probe execution results
    ProbeResults {
        probes: Vec<ProbeResultSnippet>,
        total_ms: u64,
    },
    /// Senior verdict data
    SeniorVerdict {
        verdict: String,
        confidence: f64,
        problems: Vec<String>,
    },
    /// Error details
    ErrorDetails {
        error: String,
        recoverable: bool,
    },
    /// Final answer summary
    AnswerSummary {
        confidence: String,
        score: f64,
        iterations_used: usize,
    },
    /// Stream metadata (v0.72.0: full question, no truncation)
    StreamMeta {
        question: String,
        junior_model: String,
        senior_model: String,
    },
    /// Generic key-value data
    KeyValue {
        pairs: Vec<(String, String)>,
    },
    /// v0.76.2: Full LLM prompt being sent (NO TRUNCATION)
    LlmPrompt {
        role: String,           // "junior" or "senior"
        model: String,          // model name
        system_prompt: String,  // FULL system prompt
        user_prompt: String,    // FULL user prompt
    },
    /// v0.76.2: Full LLM response received (NO TRUNCATION)
    LlmResponse {
        role: String,           // "junior" or "senior"
        model: String,          // model name
        response: String,       // FULL response
        elapsed_ms: u64,        // how long it took
    },
}

impl DebugEventData {
    /// Format data with indentation for terminal display
    pub fn format_indented(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let dim = "\x1b[2m";
        let reset = "\x1b[0m";
        let cyan = "\x1b[38;2;0;255;255m";

        match self {
            Self::JuniorPlan { intent, probes_requested, has_draft } => {
                let mut out = format!("{pad}{dim}Intent:{reset} {intent}\n");
                if !probes_requested.is_empty() {
                    out.push_str(&format!("{pad}{dim}Probes:{reset} {}\n", probes_requested.join(", ")));
                }
                out.push_str(&format!("{pad}{dim}Has draft:{reset} {}\n", if *has_draft { "yes" } else { "no" }));
                out
            }
            Self::AnnaProbe { probe_id, command, latency_ms, success } => {
                let status = if *success {
                    "\x1b[38;2;0;255;0m[OK]\x1b[0m"
                } else {
                    "\x1b[38;2;255;0;0m[FAIL]\x1b[0m"
                };
                format!(
                    "{pad}{status} {cyan}{probe_id}{reset} ({dim}{latency_ms}ms{reset})\n\
                     {pad}{dim}cmd:{reset} {command}\n"
                )
            }
            Self::ProbeResults { probes, total_ms } => {
                let mut out = String::new();
                for probe in probes {
                    let status = if probe.success {
                        "\x1b[38;2;0;255;0m[OK]\x1b[0m"
                    } else {
                        "\x1b[38;2;255;0;0m[FAIL]\x1b[0m"
                    };
                    out.push_str(&format!(
                        "{pad}{status} {cyan}{}{reset} ({dim}{}ms{reset})\n",
                        probe.probe_id, probe.latency_ms
                    ));
                    if !probe.snippet.is_empty() {
                        // Truncate and indent snippet
                        let snippet = truncate_str(&probe.snippet, 120);
                        out.push_str(&format!("{pad}  {dim}-> {snippet}{reset}\n"));
                    }
                }
                out.push_str(&format!("{pad}{dim}Total probe time: {total_ms}ms{reset}\n"));
                out
            }
            Self::SeniorVerdict { verdict, confidence, problems } => {
                let verdict_color = match verdict.as_str() {
                    "approve" | "fix_and_accept" => "\x1b[38;2;0;255;0m",
                    "needs_more_probes" => "\x1b[38;2;255;165;0m",
                    "refuse" => "\x1b[38;2;255;0;0m",
                    _ => "\x1b[0m",
                };
                let mut out = format!(
                    "{pad}{dim}Verdict:{reset} {verdict_color}{verdict}{reset}\n"
                );
                out.push_str(&format!("{pad}{dim}Confidence:{reset} {:.1}%\n", confidence * 100.0));
                if !problems.is_empty() {
                    out.push_str(&format!("{pad}{dim}Issues:{reset}\n"));
                    for p in problems.iter().take(3) {
                        out.push_str(&format!("{pad}  - {}\n", truncate_str(p, 80)));
                    }
                }
                out
            }
            Self::ErrorDetails { error, recoverable } => {
                format!(
                    "{pad}{dim}Error:{reset} {error}\n{pad}{dim}Recoverable:{reset} {}\n",
                    if *recoverable { "yes" } else { "no" }
                )
            }
            Self::AnswerSummary { confidence, score, iterations_used } => {
                let conf_color = match confidence.as_str() {
                    "GREEN" => "\x1b[38;2;0;255;0m",
                    "YELLOW" => "\x1b[38;2;255;215;0m",
                    "RED" => "\x1b[38;2;255;0;0m",
                    _ => "\x1b[0m",
                };
                format!(
                    "{pad}{dim}Confidence:{reset} {conf_color}{confidence}{reset}\n\
                     {pad}{dim}Score:{reset} {:.2}\n\
                     {pad}{dim}Iterations:{reset} {}\n",
                    score, iterations_used
                )
            }
            Self::StreamMeta { question, junior_model, senior_model } => {
                // v0.72.0: Full question, no truncation
                format!(
                    "{pad}{dim}Question:{reset} {question}\n\
                     {pad}{dim}Junior:{reset} {junior_model}\n\
                     {pad}{dim}Senior:{reset} {senior_model}\n"
                )
            }
            Self::KeyValue { pairs } => {
                let mut out = String::new();
                for (k, v) in pairs {
                    out.push_str(&format!("{pad}{dim}{}:{reset} {}\n", k, v));
                }
                out
            }
            // v0.76.2: Full LLM prompt display - dialog style
            Self::LlmPrompt { role, model, system_prompt, user_prompt } => {
                let role_color = if role == "junior" {
                    "\x1b[38;2;100;149;237m" // Cornflower blue
                } else {
                    "\x1b[38;2;255;165;0m" // Orange
                };
                let mut out = format!(
                    "\n{pad}╔══════════════════════════════════════════════════════════════════════════════╗\n\
                     {pad}║  {role_color}[{} -> {}]{reset}\n\
                     {pad}╚══════════════════════════════════════════════════════════════════════════════╝\n",
                    role.to_uppercase(), model
                );
                out.push_str(&format!("{pad}{dim}SYSTEM PROMPT ({} chars):{reset}\n", system_prompt.len()));
                out.push_str(&format!("{pad}{cyan}{}{reset}\n", system_prompt));
                out.push_str(&format!("{pad}{dim}USER PROMPT ({} chars):{reset}\n", user_prompt.len()));
                out.push_str(&format!("{pad}{cyan}{}{reset}\n", user_prompt));
                out
            }
            // v0.76.2: Full LLM response display - dialog style
            Self::LlmResponse { role, model, response, elapsed_ms } => {
                let role_color = if role == "junior" {
                    "\x1b[38;2;100;149;237m" // Cornflower blue
                } else {
                    "\x1b[38;2;255;165;0m" // Orange
                };
                let mut out = format!(
                    "\n{pad}╔══════════════════════════════════════════════════════════════════════════════╗\n\
                     {pad}║  {role_color}[{} <- {}]{reset} ({elapsed_ms}ms)\n\
                     {pad}╚══════════════════════════════════════════════════════════════════════════════╝\n",
                    role.to_uppercase(), model
                );
                out.push_str(&format!("{pad}{dim}RESPONSE ({} chars):{reset}\n", response.len()));
                out.push_str(&format!("{pad}{cyan}{}{reset}\n", response));
                out
            }
        }
    }
}

/// Snippet of probe execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResultSnippet {
    /// Probe identifier
    pub probe_id: String,
    /// Success status
    pub success: bool,
    /// Execution latency in ms
    pub latency_ms: u64,
    /// Output snippet (truncated)
    pub snippet: String,
}

/// Debug stream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugStreamConfig {
    /// Whether live debug streaming is enabled
    pub enabled: bool,
    /// Maximum snippet length in characters
    pub max_snippet_len: usize,
    /// Whether to include raw LLM prompts/responses
    pub include_raw_llm: bool,
    /// Flush interval in milliseconds (0 = immediate)
    pub flush_interval_ms: u64,
}

impl Default for DebugStreamConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_snippet_len: 200,
            include_raw_llm: false,
            flush_interval_ms: 0, // Immediate flush
        }
    }
}

impl DebugStreamConfig {
    /// Create config from environment and settings
    pub fn from_env() -> Self {
        let enabled = std::env::var("ANNA_DEBUG")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        Self {
            enabled,
            ..Default::default()
        }
    }
}

/// Debug event emitter trait for streaming
pub trait DebugEventEmitter: Send + Sync {
    /// Emit a debug event
    fn emit(&self, event: DebugEvent);

    /// Check if debug mode is active
    fn is_active(&self) -> bool;

    /// Emit iteration started
    fn iteration_started(&self, iteration: usize) {
        if self.is_active() {
            self.emit(DebugEvent::new(
                DebugEventType::IterationStarted,
                iteration,
                format!("Starting iteration {}", iteration),
            ));
        }
    }

    /// Emit junior plan started
    fn junior_plan_started(&self, iteration: usize) {
        if self.is_active() {
            self.emit(DebugEvent::new(
                DebugEventType::JuniorPlanStarted,
                iteration,
                "Junior LLM analyzing question and planning probes",
            ));
        }
    }

    /// Emit junior plan done
    fn junior_plan_done(&self, iteration: usize, intent: &str, probes: &[String], has_draft: bool) {
        if self.is_active() {
            self.emit(DebugEvent::new(
                DebugEventType::JuniorPlanDone,
                iteration,
                format!("Junior identified intent: {}", intent),
            ).with_data(DebugEventData::JuniorPlan {
                intent: intent.to_string(),
                probes_requested: probes.to_vec(),
                has_draft,
            }));
        }
    }

    /// Emit single probe execution (v0.72.0)
    fn anna_probe(&self, iteration: usize, probe_id: &str, command: &str, latency_ms: u64, success: bool) {
        if self.is_active() {
            let status = if success { "ok" } else { "failed" };
            self.emit(DebugEvent::new(
                DebugEventType::AnnaProbe,
                iteration,
                format!("Ran {}: {}", probe_id, status),
            ).with_data(DebugEventData::AnnaProbe {
                probe_id: probe_id.to_string(),
                command: command.to_string(),
                latency_ms,
                success,
            }));
        }
    }

    /// Emit probes executed
    fn probes_executed(&self, iteration: usize, results: Vec<ProbeResultSnippet>, total_ms: u64) {
        if self.is_active() {
            let count = results.len();
            let success_count = results.iter().filter(|r| r.success).count();
            self.emit(DebugEvent::new(
                DebugEventType::ProbesExecuted,
                iteration,
                format!("Executed {} probes ({} succeeded)", count, success_count),
            ).with_data(DebugEventData::ProbeResults {
                probes: results,
                total_ms,
            }));
        }
    }

    /// Emit senior review started
    fn senior_review_started(&self, iteration: usize) {
        if self.is_active() {
            self.emit(DebugEvent::new(
                DebugEventType::SeniorReviewStarted,
                iteration,
                "Senior LLM reviewing draft and evidence",
            ));
        }
    }

    /// Emit senior review done
    fn senior_review_done(&self, iteration: usize, verdict: &str, confidence: f64, problems: &[String]) {
        if self.is_active() {
            self.emit(DebugEvent::new(
                DebugEventType::SeniorReviewDone,
                iteration,
                format!("Senior verdict: {} (confidence: {:.0}%)", verdict, confidence * 100.0),
            ).with_data(DebugEventData::SeniorVerdict {
                verdict: verdict.to_string(),
                confidence,
                problems: problems.to_vec(),
            }));
        }
    }

    /// Emit retry started
    fn retry_started(&self, iteration: usize, reason: &str) {
        if self.is_active() {
            self.emit(DebugEvent::new(
                DebugEventType::RetryStarted,
                iteration,
                format!("Retrying: {}", reason),
            ));
        }
    }

    /// Emit answer ready
    fn answer_ready(&self, confidence: &str, score: f64, iterations: usize) {
        if self.is_active() {
            self.emit(DebugEvent::new(
                DebugEventType::AnswerReady,
                0,
                format!("Answer ready with {} confidence", confidence),
            ).with_data(DebugEventData::AnswerSummary {
                confidence: confidence.to_string(),
                score,
                iterations_used: iterations,
            }));
        }
    }

    /// Emit error
    fn error(&self, error: &str, recoverable: bool) {
        if self.is_active() {
            self.emit(DebugEvent::new(
                DebugEventType::Error,
                0,
                truncate_str(error, 100).to_string(),
            ).with_data(DebugEventData::ErrorDetails {
                error: error.to_string(),
                recoverable,
            }));
        }
    }

    /// v0.77.0: Emit LLM prompt being sent (full dialog view in annactl)
    fn llm_prompt_sent(&self, iteration: usize, role: &str, model: &str, system_prompt: &str, user_prompt: &str) {
        if self.is_active() {
            self.emit(DebugEvent::new(
                DebugEventType::LlmPromptSent,
                iteration,
                format!("Sending prompt to {} ({})", role, model),
            ).with_data(DebugEventData::LlmPrompt {
                role: role.to_string(),
                model: model.to_string(),
                system_prompt: system_prompt.to_string(),
                user_prompt: user_prompt.to_string(),
            }));
        }
    }

    /// v0.77.0: Emit LLM response received (full dialog view in annactl)
    fn llm_response_received(&self, iteration: usize, role: &str, model: &str, response: &str, elapsed_ms: u64) {
        if self.is_active() {
            self.emit(DebugEvent::new(
                DebugEventType::LlmResponseReceived,
                iteration,
                format!("{} responded in {}ms", role, elapsed_ms),
            ).with_data(DebugEventData::LlmResponse {
                role: role.to_string(),
                model: model.to_string(),
                response: response.to_string(),
                elapsed_ms,
            }));
        }
    }
}

/// Helper: truncate string with ellipsis
fn truncate_str(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        let end = s.char_indices()
            .take_while(|(i, _)| *i < max_len - 3)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(max_len - 3);
        &s[..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_event_serialization() {
        let event = DebugEvent::new(
            DebugEventType::JuniorPlanDone,
            1,
            "Junior finished planning",
        ).with_data(DebugEventData::JuniorPlan {
            intent: "system_info".to_string(),
            probes_requested: vec!["mem.info".to_string()],
            has_draft: true,
        });

        let json = event.to_ndjson();
        assert!(json.contains("JUNIOR_PLAN_DONE"));
        assert!(json.contains("system_info"));
    }

    #[test]
    fn test_event_type_labels() {
        assert_eq!(DebugEventType::JuniorPlanStarted.as_label(), "JUNIOR: PLAN");
        assert_eq!(DebugEventType::SeniorReviewDone.as_label(), "SENIOR: VERDICT");
        assert_eq!(DebugEventType::ProbesExecuted.as_label(), "ANNA: PROBES");
        assert_eq!(DebugEventType::AnnaProbe.as_label(), "ANNA: PROBE");
    }

    #[test]
    fn test_terminal_formatting() {
        let event = DebugEvent::new(
            DebugEventType::IterationStarted,
            1,
            "Starting iteration 1",
        );

        let output = event.format_terminal();
        assert!(output.contains("ITERATION"));  // v0.72.0: Label is now just "ITERATION"
        assert!(output.contains("iter 1"));
    }

    #[test]
    fn test_probe_results_formatting() {
        let data = DebugEventData::ProbeResults {
            probes: vec![
                ProbeResultSnippet {
                    probe_id: "mem.info".to_string(),
                    success: true,
                    latency_ms: 42,
                    snippet: "MemTotal: 16GB".to_string(),
                },
            ],
            total_ms: 42,
        };

        let formatted = data.format_indented(2);
        assert!(formatted.contains("mem.info"));
        assert!(formatted.contains("42ms"));
    }

    #[test]
    fn test_config_from_env() {
        // Default should be disabled
        let config = DebugStreamConfig::default();
        assert!(!config.enabled);
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8).len(), 5); // "hello" (5 chars before "...")
    }
}
