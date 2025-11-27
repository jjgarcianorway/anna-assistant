//! Logging Module v0.8.0
//!
//! Unified logging subsystem for Anna with:
//! - Multiple log levels (TRACE, DEBUG, INFO, WARN, ERROR)
//! - JSONL structured file output
//! - Request correlation via request_id
//! - Redaction and truncation for secrets
//! - Log rotation support

pub mod config;
pub mod redaction;
pub mod request;
pub mod research_trace;
pub mod writer;

pub use config::*;
pub use redaction::*;
pub use request::*;
pub use research_trace::*;
pub use writer::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Log level enum
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace = 0,
    #[default]
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" | "warning" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

/// Log component category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogComponent {
    Daemon,
    Probe,
    Llm,
    Ipc,
    Request,
    Repair,
    Update,
    Config,
    SelfHealth,
    Research,
}

impl LogComponent {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogComponent::Daemon => "daemon",
            LogComponent::Probe => "probe",
            LogComponent::Llm => "llm",
            LogComponent::Ipc => "ipc",
            LogComponent::Request => "request",
            LogComponent::Repair => "repair",
            LogComponent::Update => "update",
            LogComponent::Config => "config",
            LogComponent::SelfHealth => "self_health",
            LogComponent::Research => "research",
        }
    }
}

/// A structured log entry (JSONL format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// ISO 8601 timestamp with timezone
    pub timestamp: DateTime<Utc>,
    /// Log level
    pub level: LogLevel,
    /// Component category
    pub component: LogComponent,
    /// Request correlation ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Human-readable message
    pub message: String,
    /// Additional structured fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<serde_json::Value>,
}

impl LogEntry {
    pub fn new(level: LogLevel, component: LogComponent, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            component,
            request_id: None,
            message: message.into(),
            fields: None,
        }
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_fields(mut self, fields: serde_json::Value) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Serialize to JSONL (single line JSON)
    pub fn to_jsonl(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                r#"{{"timestamp":"{}","level":"{}","component":"{}","message":"serialization_error"}}"#,
                self.timestamp.to_rfc3339(),
                self.level.as_str(),
                self.component.as_str()
            )
        })
    }
}

/// Request trace summary for anna-requests.log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTrace {
    pub request_id: String,
    pub timestamp_start: DateTime<Utc>,
    pub timestamp_end: DateTime<Utc>,
    pub duration_ms: u64,
    pub user_query: String,
    pub probe_summary: Vec<String>,
    pub self_health_actions: Vec<String>,
    pub reliability_score: f64,
    pub result_status: RequestStatus,
}

/// Request result status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequestStatus {
    Ok,
    Degraded,
    Failed,
}

/// LLM orchestration trace for anna-llm.log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTrace {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub phase: LlmPhase,
    pub user_query_summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probes_executed: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_summary: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reliability_breakdown: Option<ReliabilityBreakdown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_answer_status: Option<String>,
}

/// LLM orchestration phase
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmPhase {
    Planning,
    Execution,
    Supervision,
    Final,
}

/// Reliability score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityBreakdown {
    pub evidence_score: f64,
    pub reasoning_score: f64,
    pub coverage_score: f64,
    pub overall_score: f64,
}

/// Global sequence number for request IDs
static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Generate a unique request ID
pub fn generate_request_id() -> String {
    let seq = REQUEST_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    let timestamp = Utc::now().timestamp_millis();
    let hash = (timestamp ^ (seq as i64)) & 0xFFFFFF;
    format!("req-{:06x}-{:04x}", hash, seq & 0xFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_log_level_parse() {
        assert_eq!(LogLevel::parse("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::parse("DEBUG"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::parse("warn"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::parse("warning"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::parse("invalid"), None);
    }

    #[test]
    fn test_log_entry_to_jsonl() {
        let entry = LogEntry::new(LogLevel::Info, LogComponent::Daemon, "Test message");
        let jsonl = entry.to_jsonl();
        assert!(jsonl.contains("\"level\":\"info\""));
        assert!(jsonl.contains("\"component\":\"daemon\""));
        assert!(jsonl.contains("\"message\":\"Test message\""));
    }

    #[test]
    fn test_log_entry_with_request_id() {
        let entry = LogEntry::new(LogLevel::Debug, LogComponent::Probe, "Running probe")
            .with_request_id("req-123456-0001");
        let jsonl = entry.to_jsonl();
        assert!(jsonl.contains("\"request_id\":\"req-123456-0001\""));
    }

    #[test]
    fn test_generate_request_id() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        assert!(id1.starts_with("req-"));
        assert!(id2.starts_with("req-"));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_request_status_serialization() {
        let status = RequestStatus::Ok;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"ok\"");
    }
}
