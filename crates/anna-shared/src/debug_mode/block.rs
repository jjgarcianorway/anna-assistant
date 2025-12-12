//! Debug block footer (v0.0.444).
//!
//! Standardized debug output appended to responses at debug level 1+.

use super::reason_codes::{ReasonCode, ReasonCodes};
use super::sanitize::{SanitizeResult, Sanitizer};
use crate::reliability_metrics::CanonicalOutcome;
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

/// Complete debug block for appending to responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DebugBlock {
    /// Request ID
    pub request_id: String,
    /// Canonical outcome
    pub outcome: Option<CanonicalOutcome>,
    /// Routed topic/domain
    pub routed_topic: String,
    /// Models used
    pub models: ModelsUsedDebug,
    /// Probes required
    pub probes_required: Vec<String>,
    /// Probes run with status
    pub probes_run: Vec<ProbeDebugInfo>,
    /// Timing breakdown
    pub timings: TimingDebug,
    /// Evidence info
    pub evidence: EvidenceDebug,
    /// Reason codes
    pub reason_codes: ReasonCodes,
    /// Translator decision (for routing transparency)
    pub translator_decision: Option<TranslatorDecision>,
    /// LLM calls (FULL mode only)
    pub llm_calls: Vec<LlmCallDebug>,
    /// Timeout info (if any)
    pub timeout: Option<TimeoutDebug>,
}

impl DebugBlock {
    /// Create new debug block.
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            ..Default::default()
        }
    }

    /// Set outcome.
    pub fn with_outcome(mut self, outcome: CanonicalOutcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    /// Set routed topic.
    pub fn with_topic(mut self, topic: &str) -> Self {
        self.routed_topic = topic.to_string();
        self
    }

    /// Add reason code.
    pub fn add_reason(&mut self, code: ReasonCode) {
        self.reason_codes.add(code);
    }

    /// Format for SUMMARY level (level 1).
    /// Shows: domain, intent, probes, outcome, reliability score, failure reason.
    pub fn format_summary(&self) -> String {
        let mut out = String::new();
        out.push_str("\n[summary]\n");
        out.push_str(&format!("  request_id          {}\n", self.request_id));
        out.push_str(&format!(
            "  outcome             {}\n",
            self.outcome.map(|o| o.label()).unwrap_or("PENDING")
        ));
        out.push_str(&format!("  routed_topic        {}\n", self.routed_topic));
        out.push_str(&format!(
            "  probes_required     [{}]\n",
            self.probes_required.join(", ")
        ));

        // Just show probe IDs and status
        if !self.probes_run.is_empty() {
            let probe_summary: Vec<String> = self
                .probes_run
                .iter()
                .map(|p| format!("{}(exit={})", p.id, p.exit_code))
                .collect();
            out.push_str(&format!(
                "  probes_run          [{}]\n",
                probe_summary.join(", ")
            ));
        }

        // Reliability/confidence score
        if !self.reason_codes.is_empty() {
            out.push_str(&format!(
                "  reason_codes        {}\n",
                self.reason_codes.display()
            ));
        }

        // Timeout info (important for failures)
        if let Some(t) = &self.timeout {
            out.push_str(&format!("  timeout             {}\n", t.stage));
        }

        out
    }

    /// Format for TRACE level (level 2).
    /// Shows: probe commands, exit codes, parsed values, LLM tokens, gate report.
    pub fn format_trace(&self) -> String {
        let mut out = String::new();
        out.push_str("\n[debug]\n");
        out.push_str(&format!("  request_id          {}\n", self.request_id));
        out.push_str(&format!(
            "  outcome             {}\n",
            self.outcome.map(|o| o.label()).unwrap_or("PENDING")
        ));
        out.push_str(&format!("  routed_topic        {}\n", self.routed_topic));
        out.push_str(&format!(
            "  models_used         {}\n",
            self.models.display()
        ));
        out.push_str(&format!(
            "  probes_required     [{}]\n",
            self.probes_required.join(", ")
        ));

        out.push_str("  probes_run:\n");
        for p in &self.probes_run {
            out.push_str(&p.display());
            out.push('\n');
        }

        out.push_str(&format!(
            "  timings             {}\n",
            self.timings.display()
        ));
        out.push_str(&format!(
            "  evidence            {}\n",
            self.evidence.display()
        ));
        out.push_str(&format!(
            "  reason_codes        {}\n",
            self.reason_codes.display()
        ));

        // Translator decision (routing transparency)
        if let Some(td) = &self.translator_decision {
            out.push_str("  translator_decision:\n");
            out.push_str(&td.display_trace());
            out.push('\n');
        }

        // Timeout info
        if let Some(t) = &self.timeout {
            out.push_str(&format!("  timeout             {}\n", t.display()));
        }

        out
    }

    /// Format for FULL level (level 3).
    /// Shows: full prompts/responses, raw probe output, parser errors.
    pub fn format_full(&self) -> String {
        let mut out = self.format_trace();

        // Add probe outputs
        if !self.probes_run.is_empty() {
            out.push_str("\n[probe_outputs]\n");
            for p in &self.probes_run {
                out.push_str(&format!("  --- {} ({}) ---\n", p.id, p.command));
                if let Some(stdout) = &p.stdout {
                    out.push_str("  stdout:\n");
                    for line in stdout.lines() {
                        out.push_str(&format!("    {}\n", line));
                    }
                }
                if let Some(stderr) = &p.stderr {
                    out.push_str("  stderr:\n");
                    for line in stderr.lines() {
                        out.push_str(&format!("    {}\n", line));
                    }
                }
            }
        }

        // Add LLM calls
        if !self.llm_calls.is_empty() {
            out.push_str("\n[llm_calls]\n");
            for call in &self.llm_calls {
                out.push_str(&format!("  --- {} ({}) ---\n", call.role, call.model));
                out.push_str(&format!("  duration: {}ms\n", call.duration_ms));
                out.push_str(&format!("  parse_success: {}\n", call.parse_success));
                if let Some(err) = &call.parse_error {
                    out.push_str(&format!("  parse_error: {}\n", err));
                }
                out.push_str("  prompt:\n");
                for line in call.prompt.lines().take(50) {
                    out.push_str(&format!("    {}\n", line));
                }
                out.push_str("  response:\n");
                for line in call.response.lines().take(50) {
                    out.push_str(&format!("    {}\n", line));
                }
            }
        }

        // Translator raw JSON
        if let Some(td) = &self.translator_decision {
            if let Some(raw) = &td.raw_json {
                out.push_str("\n[translator_raw]\n");
                out.push_str(raw);
                out.push('\n');
            }
        }

        out
    }

    /// Format based on level.
    pub fn format(&self, level: super::DebugLevel) -> Option<String> {
        match level {
            super::DebugLevel::Off => None,
            super::DebugLevel::Summary => Some(self.format_summary()),
            super::DebugLevel::Trace => Some(self.format_trace()),
            super::DebugLevel::Full => Some(self.format_full()),
        }
    }

    /// Generate user-facing timeout message.
    pub fn timeout_user_message(&self) -> Option<String> {
        self.timeout.as_ref().map(|t| {
            format!(
                "I failed due to LLM timeout at {}. No verified answer produced.",
                t.stage
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_block_trace() {
        let mut block = DebugBlock::new("REQ-001")
            .with_outcome(CanonicalOutcome::AnsweredVerified)
            .with_topic("storage");

        block.models = ModelsUsedDebug {
            translator: Some("qwen2.5:3b".into()),
            specialist: Some("qwen2.5:7b".into()),
            verifier: None,
        };

        block.probes_required = vec!["df".into(), "du".into()];
        block
            .probes_run
            .push(ProbeDebugInfo::new("df", "df -h", 0, 50));

        block.timings = TimingDebug {
            total_ms: 1500,
            probe_ms: 100,
            llm_ms: 1200,
            translator_ms: 200,
            specialist_ms: 1000,
        };

        block.add_reason(ReasonCode::Success);

        let trace = block.format_trace();
        assert!(trace.contains("REQ-001"));
        assert!(trace.contains("VERIFIED"));
        assert!(trace.contains("storage"));
        assert!(trace.contains("qwen2.5:3b"));
        assert!(trace.contains("df"));
        assert!(trace.contains("SUCCESS"));
    }

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
