//! Debug Trace Types - v0.0.443.

use serde::{Deserialize, Serialize};
use super::trace::{redact_sensitive, format_json_indented};

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
