//! Debug Trace Observability (Part 2) - v0.0.443.
//!
//! Structured debug log per request:
//! - Path: /var/lib/anna/debug/<request_id>.jsonl
//! - Each line is a JSON event with stage, model, input, output
//!
//! `annactl trace <request_id>` renders readable view.

// Re-export all public types from sibling modules
pub use super::trace_types::{DebugSummary, RequestTrace, TraceEvent, TraceStage};
pub use super::trace_file_manager::TraceFileManager;

/// Redact sensitive fields from JSON.
pub(crate) fn redact_sensitive(value: &serde_json::Value) -> serde_json::Value {
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

/// Format JSON with indentation.
pub(crate) fn format_json_indented(value: &serde_json::Value, indent: usize) -> String {
    let pretty = serde_json::to_string_pretty(value).unwrap_or_default();
    let prefix = " ".repeat(indent);
    pretty
        .lines()
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
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
