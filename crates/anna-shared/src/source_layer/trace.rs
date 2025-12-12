//! Debug Trace Observability (Part 2) - v0.0.443.
//!
//! Structured debug log per request:
//! - Path: /var/lib/anna/debug/<request_id>.jsonl
//! - Each line is a JSON event with stage, model, input, output
//!
//! `annactl trace <request_id>` renders readable view.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Debug trace event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Timestamp (ISO 8601).
    pub ts: String,
    /// Request ID.
    pub request_id: String,
    /// Pipeline stage.
    pub stage: TraceStage,
    /// Model used (if any).
    pub model: Option<String>,
    /// Input to this stage.
    pub input: serde_json::Value,
    /// Output from this stage.
    pub output: serde_json::Value,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Redacted fields.
    #[serde(default)]
    pub redactions: Vec<String>,
}

/// Pipeline stage for tracing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceStage {
    /// User query received.
    Query,
    /// Intent translation.
    Translator,
    /// Facts collection.
    Facts,
    /// Probe execution.
    Probes,
    /// Source fetching (man/wiki/help).
    Sources,
    /// Research planning.
    Planner,
    /// Specialist reasoning.
    Specialist,
    /// Supervisor review.
    Supervisor,
    /// Answer rendering.
    Renderer,
    /// Final response.
    Response,
}

impl TraceStage {
    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Query => "QUERY",
            Self::Translator => "TRANSLATOR",
            Self::Facts => "FACTS",
            Self::Probes => "PROBES",
            Self::Sources => "SOURCES",
            Self::Planner => "PLANNER",
            Self::Specialist => "SPECIALIST",
            Self::Supervisor => "SUPERVISOR",
            Self::Renderer => "RENDERER",
            Self::Response => "RESPONSE",
        }
    }
}

impl TraceEvent {
    /// Create new event.
    pub fn new(request_id: &str, stage: TraceStage) -> Self {
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            request_id: request_id.to_string(),
            stage,
            model: None,
            input: serde_json::Value::Null,
            output: serde_json::Value::Null,
            duration_ms: 0,
            redactions: Vec::new(),
        }
    }

    /// Set model.
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    /// Set input.
    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = redact_sensitive(&input);
        self
    }

    /// Set output.
    pub fn with_output(mut self, output: serde_json::Value) -> Self {
        self.output = redact_sensitive(&output);
        self
    }

    /// Set duration.
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Serialize to JSON line.
    pub fn to_jsonl(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
}

/// Redact sensitive fields from JSON.
fn redact_sensitive(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, val) in map {
                if is_sensitive_key(key) {
                    new_map.insert(
                        key.clone(),
                        serde_json::Value::String("[REDACTED]".to_string()),
                    );
                } else {
                    new_map.insert(key.clone(), redact_sensitive(val));
                }
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(redact_sensitive).collect())
        }
        serde_json::Value::String(s) if looks_like_secret(s) => {
            serde_json::Value::String("[REDACTED]".to_string())
        }
        other => other.clone(),
    }
}

/// Check if a key name is sensitive.
fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("token")
        || lower.contains("key")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("auth")
        || lower.contains("credential")
}

/// Check if a string value looks like a secret.
fn looks_like_secret(s: &str) -> bool {
    // SSH private keys
    if s.contains("-----BEGIN") && (s.contains("PRIVATE KEY") || s.contains("RSA")) {
        return true;
    }

    // API keys (long alphanumeric strings)
    if s.len() > 30
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return true;
    }

    // Bearer tokens
    if s.starts_with("Bearer ") || s.starts_with("sk-") || s.starts_with("ghp_") {
        return true;
    }

    false
}

/// Request trace containing all events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTrace {
    /// Request ID.
    pub request_id: String,
    /// Events in order.
    pub events: Vec<TraceEvent>,
    /// Final outcome.
    pub outcome: Option<String>,
    /// Total duration.
    pub total_duration_ms: u64,
}

impl RequestTrace {
    /// Create new trace.
    pub fn new(request_id: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            events: Vec::new(),
            outcome: None,
            total_duration_ms: 0,
        }
    }

    /// Add event.
    pub fn add_event(&mut self, event: TraceEvent) {
        self.total_duration_ms += event.duration_ms;
        self.events.push(event);
    }

    /// Set outcome.
    pub fn set_outcome(&mut self, outcome: &str) {
        self.outcome = Some(outcome.to_string());
    }

    /// Get event by stage.
    pub fn get_stage(&self, stage: TraceStage) -> Option<&TraceEvent> {
        self.events.iter().find(|e| e.stage == stage)
    }

    /// Render readable trace output.
    pub fn render(&self) -> String {
        let mut output = format!("=== Request Trace: {} ===\n\n", self.request_id);

        for event in &self.events {
            output.push_str(&format!(
                "[{}] {} ({}ms)\n",
                event.ts,
                event.stage.label(),
                event.duration_ms
            ));

            if let Some(ref model) = event.model {
                output.push_str(&format!("  Model: {}\n", model));
            }

            if event.input != serde_json::Value::Null {
                output.push_str("  Input:\n");
                output.push_str(&format_json_indented(&event.input, 4));
                output.push('\n');
            }

            if event.output != serde_json::Value::Null {
                output.push_str("  Output:\n");
                output.push_str(&format_json_indented(&event.output, 4));
                output.push('\n');
            }

            output.push('\n');
        }

        if let Some(ref outcome) = self.outcome {
            output.push_str(&format!("Outcome: {}\n", outcome));
        }

        output.push_str(&format!("Total Duration: {}ms\n", self.total_duration_ms));

        output
    }
}

/// Format JSON with indentation.
fn format_json_indented(value: &serde_json::Value, indent: usize) -> String {
    let pretty = serde_json::to_string_pretty(value).unwrap_or_default();
    let prefix = " ".repeat(indent);
    pretty
        .lines()
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Console debug summary (short).
#[derive(Debug, Clone)]
pub struct DebugSummary {
    /// Request ID.
    pub request_id: String,
    /// Intent detected.
    pub intent: String,
    /// Domain.
    pub domain: String,
    /// Probes executed.
    pub probes: Vec<String>,
    /// Sources fetched.
    pub sources: Vec<String>,
    /// Model used.
    pub model: String,
    /// Final state.
    pub state: String,
}

impl DebugSummary {
    /// Format for console output.
    pub fn display(&self) -> String {
        format!(
            "request_id={} intent={} domain={} probes=[{}] sources=[{}] model={} state={}",
            self.request_id,
            self.intent,
            self.domain,
            self.probes.join(","),
            self.sources.join(","),
            self.model,
            self.state
        )
    }
}

/// Trace file manager.
pub struct TraceFileManager {
    /// Debug directory.
    debug_dir: String,
}

impl TraceFileManager {
    /// Default debug directory.
    pub const DEFAULT_DIR: &'static str = "/var/lib/anna/debug";

    /// Create new manager.
    pub fn new() -> Self {
        Self {
            debug_dir: Self::DEFAULT_DIR.to_string(),
        }
    }

    /// Create with custom directory.
    pub fn with_dir(dir: &str) -> Self {
        Self {
            debug_dir: dir.to_string(),
        }
    }

    /// Get trace file path for request.
    pub fn trace_path(&self, request_id: &str) -> String {
        format!("{}/{}.jsonl", self.debug_dir, request_id)
    }

    /// Write event to trace file.
    pub fn write_event(&self, event: &TraceEvent) -> Result<(), String> {
        // Ensure directory exists
        std::fs::create_dir_all(&self.debug_dir)
            .map_err(|e| format!("Failed to create debug dir: {}", e))?;

        let path = self.trace_path(&event.request_id);
        let line = event.to_jsonl()?;

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Failed to open trace file: {}", e))?;

        writeln!(file, "{}", line).map_err(|e| format!("Failed to write trace: {}", e))
    }

    /// Read trace for request.
    pub fn read_trace(&self, request_id: &str) -> Result<RequestTrace, String> {
        let path = self.trace_path(request_id);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read trace file: {}", e))?;

        let mut trace = RequestTrace::new(request_id);

        for line in content.lines() {
            if let Ok(event) = serde_json::from_str::<TraceEvent>(line) {
                trace.add_event(event);
            }
        }

        Ok(trace)
    }

    /// List available traces.
    pub fn list_traces(&self) -> Result<Vec<String>, String> {
        let entries = std::fs::read_dir(&self.debug_dir)
            .map_err(|e| format!("Failed to read debug dir: {}", e))?;

        let mut ids = Vec::new();
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".jsonl") {
                    ids.push(name.trim_end_matches(".jsonl").to_string());
                }
            }
        }

        ids.sort();
        ids.reverse(); // Most recent first
        Ok(ids)
    }
}

impl Default for TraceFileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_event() {
        let event = TraceEvent::new("req-123", TraceStage::Translator)
            .with_model("qwen2.5:7b")
            .with_duration(150);

        assert_eq!(event.request_id, "req-123");
        assert_eq!(event.stage, TraceStage::Translator);
        assert_eq!(event.model, Some("qwen2.5:7b".to_string()));
    }

    #[test]
    fn test_redaction() {
        let value = serde_json::json!({
            "query": "test",
            "api_key": "sk-12345",
            "data": {
                "token": "secret123"
            }
        });

        let redacted = redact_sensitive(&value);
        assert_eq!(redacted["api_key"], "[REDACTED]");
        assert_eq!(redacted["data"]["token"], "[REDACTED]");
        assert_eq!(redacted["query"], "test");
    }

    #[test]
    fn test_sensitive_detection() {
        assert!(is_sensitive_key("api_key"));
        assert!(is_sensitive_key("AUTH_TOKEN"));
        assert!(!is_sensitive_key("query"));

        assert!(looks_like_secret("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(looks_like_secret("sk-proj-abc123"));
        assert!(!looks_like_secret("hello world"));
    }

    #[test]
    fn test_request_trace_render() {
        let mut trace = RequestTrace::new("req-123");
        trace.add_event(
            TraceEvent::new("req-123", TraceStage::Query)
                .with_input(serde_json::json!({"query": "test"}))
                .with_duration(10),
        );
        trace.add_event(
            TraceEvent::new("req-123", TraceStage::Translator)
                .with_model("qwen2.5:7b")
                .with_output(serde_json::json!({"intent": "test"}))
                .with_duration(100),
        );
        trace.set_outcome("ANSWERED");

        let rendered = trace.render();
        assert!(rendered.contains("req-123"));
        assert!(rendered.contains("QUERY"));
        assert!(rendered.contains("TRANSLATOR"));
        assert!(rendered.contains("ANSWERED"));
    }

    #[test]
    fn test_debug_summary() {
        let summary = DebugSummary {
            request_id: "req-123".to_string(),
            intent: "packages.install".to_string(),
            domain: "packages".to_string(),
            probes: vec!["pacman_query".to_string()],
            sources: vec!["man:pacman(8)".to_string()],
            model: "qwen2.5:7b".to_string(),
            state: "ANSWERED".to_string(),
        };

        let display = summary.display();
        assert!(display.contains("request_id=req-123"));
        assert!(display.contains("intent=packages.install"));
    }
}
