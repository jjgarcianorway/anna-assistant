//! Debug types for debug block components.
//!
//! Contains all the individual debug info types used by DebugBlock.

use super::sanitize::Sanitizer;
use serde::{Deserialize, Serialize};

/// Translator decision for routing transparency.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslatorDecision {
    /// Detected intent
    pub intent: String,
    /// Detected domain
    pub domain: String,
    /// Selected probes
    pub probes: Vec<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Raw JSON output (for FULL mode)
    pub raw_json: Option<String>,
}

impl TranslatorDecision {
    /// Create new decision.
    pub fn new(intent: &str, domain: &str, probes: Vec<String>, confidence: f32) -> Self {
        Self {
            intent: intent.to_string(),
            domain: domain.to_string(),
            probes,
            confidence,
            raw_json: None,
        }
    }

    /// Attach raw JSON output.
    pub fn with_raw(mut self, raw: &str) -> Self {
        self.raw_json = Some(raw.to_string());
        self
    }

    /// Format for TRACE display.
    pub fn display_trace(&self) -> String {
        format!(
            "  intent: {}\n  domain: {}\n  probes: [{}]\n  confidence: {:.2}",
            self.intent,
            self.domain,
            self.probes.join(", "),
            self.confidence
        )
    }
}

/// Probe execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeStatus {
    Ok,
    Fail,
    Timeout,
    NotFound,
    Empty,
}

impl ProbeStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Fail => "fail",
            Self::Timeout => "timeout",
            Self::NotFound => "not_found",
            Self::Empty => "empty",
        }
    }
}

impl std::fmt::Display for ProbeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Debug info for a single probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeDebugInfo {
    /// Probe ID
    pub id: String,
    /// Command that was run
    pub command: String,
    /// Exit code
    pub exit_code: i32,
    /// Duration in ms
    pub duration_ms: u64,
    /// Status (ok/fail/timeout)
    pub status: ProbeStatus,
    /// Stdout (for FULL mode, sanitized)
    pub stdout: Option<String>,
    /// Stderr (for FULL mode, sanitized)
    pub stderr: Option<String>,
}

impl ProbeDebugInfo {
    /// Create from probe result.
    pub fn new(id: &str, command: &str, exit_code: i32, duration_ms: u64) -> Self {
        let status = if exit_code == 0 {
            ProbeStatus::Ok
        } else {
            ProbeStatus::Fail
        };

        Self {
            id: id.to_string(),
            command: command.to_string(),
            exit_code,
            duration_ms,
            status,
            stdout: None,
            stderr: None,
        }
    }

    /// Set status.
    pub fn with_status(mut self, status: ProbeStatus) -> Self {
        self.status = status;
        self
    }

    /// Attach output (sanitized).
    pub fn with_output(mut self, stdout: &str, stderr: &str) -> Self {
        let sanitizer = Sanitizer::default();
        self.stdout = Some(sanitizer.sanitize_probe_output(stdout).content);
        self.stderr = if stderr.is_empty() {
            None
        } else {
            Some(sanitizer.sanitize_probe_output(stderr).content)
        };
        self
    }

    /// Format for display.
    pub fn display(&self) -> String {
        format!(
            "    {} ({}) {}ms [exit {}]",
            self.id, self.status, self.duration_ms, self.exit_code
        )
    }
}

/// Models used during request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelsUsedDebug {
    pub translator: Option<String>,
    pub specialist: Option<String>,
    pub verifier: Option<String>,
}

impl ModelsUsedDebug {
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if let Some(t) = &self.translator {
            parts.push(format!("translator:{}", t));
        }
        if let Some(s) = &self.specialist {
            parts.push(format!("specialist:{}", s));
        }
        if let Some(v) = &self.verifier {
            parts.push(format!("verifier:{}", v));
        }
        if parts.is_empty() {
            "none".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Timing breakdown.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimingDebug {
    pub total_ms: u64,
    pub probe_ms: u64,
    pub llm_ms: u64,
    pub translator_ms: u64,
    pub specialist_ms: u64,
}

impl TimingDebug {
    pub fn display(&self) -> String {
        format!(
            "total:{}ms probe:{}ms llm:{}ms (translator:{}ms specialist:{}ms)",
            self.total_ms, self.probe_ms, self.llm_ms, self.translator_ms, self.specialist_ms
        )
    }
}

/// Evidence summary for debug.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceDebug {
    pub claim_count: u32,
    pub claims_with_evidence: u32,
    pub evidence_coverage: f32,
    pub evidence_ids: Vec<String>,
}

impl EvidenceDebug {
    pub fn display(&self) -> String {
        format!(
            "claims:{} with_evidence:{} coverage:{:.0}%",
            self.claim_count,
            self.claims_with_evidence,
            self.evidence_coverage * 100.0
        )
    }
}

/// LLM call debug info (for FULL mode).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCallDebug {
    /// Which LLM (translator/specialist/verifier)
    pub role: String,
    /// Model name
    pub model: String,
    /// Prompt (sanitized)
    pub prompt: String,
    /// Response (sanitized)
    pub response: String,
    /// Duration in ms
    pub duration_ms: u64,
    /// Parse result
    pub parse_success: bool,
    /// Parse error if any
    pub parse_error: Option<String>,
    /// Token count if available
    pub token_count: Option<u32>,
}

impl LlmCallDebug {
    /// Create new LLM call debug info.
    pub fn new(role: &str, model: &str) -> Self {
        Self {
            role: role.to_string(),
            model: model.to_string(),
            prompt: String::new(),
            response: String::new(),
            duration_ms: 0,
            parse_success: false,
            parse_error: None,
            token_count: None,
        }
    }

    /// Set prompt and response (will be sanitized).
    pub fn with_io(mut self, prompt: &str, response: &str, sanitizer: &Sanitizer) -> Self {
        self.prompt = sanitizer.sanitize_llm_output(prompt).content;
        self.response = sanitizer.sanitize_llm_output(response).content;
        self
    }

    /// Set timing.
    pub fn with_timing(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Set parse result.
    pub fn with_parse(mut self, success: bool, error: Option<String>) -> Self {
        self.parse_success = success;
        self.parse_error = error;
        self
    }
}

/// Timeout diagnostic info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutDebug {
    /// Stage where timeout occurred
    pub stage: String,
    /// Model that timed out
    pub model: Option<String>,
    /// Configured timeout (ms)
    pub timeout_ms: u64,
    /// Elapsed time (ms)
    pub elapsed_ms: u64,
    /// Partial output length (if any)
    pub partial_output_len: usize,
}

impl TimeoutDebug {
    pub fn new(stage: &str, timeout_ms: u64, elapsed_ms: u64) -> Self {
        Self {
            stage: stage.to_string(),
            model: None,
            timeout_ms,
            elapsed_ms,
            partial_output_len: 0,
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    pub fn with_partial(mut self, len: usize) -> Self {
        self.partial_output_len = len;
        self
    }

    pub fn display(&self) -> String {
        let model_str = self.model.as_deref().unwrap_or("unknown");
        format!(
            "Timeout at {} (model: {}, configured: {}ms, elapsed: {}ms, partial: {} chars)",
            self.stage, model_str, self.timeout_ms, self.elapsed_ms, self.partial_output_len
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translator_decision() {
        let td = TranslatorDecision::new("diagnose", "storage", vec!["df".into()], 0.85);
        let display = td.display_trace();
        assert!(display.contains("diagnose"));
        assert!(display.contains("storage"));
        assert!(display.contains("0.85"));
    }

    #[test]
    fn test_timeout_debug() {
        let timeout = TimeoutDebug::new("specialist", 10000, 15000)
            .with_model("qwen2.5:7b")
            .with_partial(500);

        let display = timeout.display();
        assert!(display.contains("specialist"));
        assert!(display.contains("10000ms"));
        assert!(display.contains("15000ms"));
        assert!(display.contains("500 chars"));
    }

    #[test]
    fn test_probe_debug_info() {
        let probe = ProbeDebugInfo::new("df", "df -h", 0, 75)
            .with_output("Filesystem Size Used\n/dev/sda 100G 50G", "");

        assert_eq!(probe.status, ProbeStatus::Ok);
        assert!(probe.stdout.is_some());
        assert!(probe.stderr.is_none());
    }
}
