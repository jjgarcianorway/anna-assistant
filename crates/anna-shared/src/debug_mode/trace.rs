//! Canonical Trace Block (v0.0.446).
//!
//! One structured block for all debug output. No scattered "internal comms".
//! Predictable layout, filterable, useful for forensics.

use super::config::DebugLevel;
use super::redact::Redactor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Route type for how the request was handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteType {
    /// Handled by deterministic probes only (no LLM)
    Deterministic,
    /// Handled by LLM specialist
    LlmSpecialist,
    /// Fell back to generic LLM response
    LlmFallback,
    /// Needed clarification from user
    Clarification,
}

impl std::fmt::Display for RouteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deterministic => write!(f, "deterministic"),
            Self::LlmSpecialist => write!(f, "llm_specialist"),
            Self::LlmFallback => write!(f, "llm_fallback"),
            Self::Clarification => write!(f, "clarification"),
        }
    }
}

/// Outcome of a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TraceOutcome {
    Success,
    FailedNoEvidence,
    FailedTimeout,
    FailedParse,
    FailedLowConfidence,
    FailedAmbiguousQuery,
    FailedContractViolation,
    FailedNoClaims,
    FailedGenericAnswer,
    FailedProbes,
}

impl std::fmt::Display for TraceOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "SUCCESS"),
            Self::FailedNoEvidence => write!(f, "FAILED_NO_EVIDENCE"),
            Self::FailedTimeout => write!(f, "FAILED_TIMEOUT"),
            Self::FailedParse => write!(f, "FAILED_PARSE"),
            Self::FailedLowConfidence => write!(f, "FAILED_LOW_CONFIDENCE"),
            Self::FailedAmbiguousQuery => write!(f, "FAILED_AMBIGUOUS_QUERY"),
            Self::FailedContractViolation => write!(f, "FAILED_CONTRACT_VIOLATION"),
            Self::FailedNoClaims => write!(f, "FAILED_NO_CLAIMS"),
            Self::FailedGenericAnswer => write!(f, "FAILED_GENERIC_ANSWER"),
            Self::FailedProbes => write!(f, "FAILED_PROBES"),
        }
    }
}

/// Probe trace info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeTrace {
    /// Probe ID
    pub id: String,
    /// Command (redacted if level < 3)
    pub command: String,
    /// Exit code
    pub exit_code: i32,
    /// Duration in ms
    pub duration_ms: u64,
    /// Parsed key-value results (level >= 2)
    pub parsed: HashMap<String, String>,
    /// Raw stdout (level 3 only, redacted)
    pub raw_stdout: Option<String>,
    /// Raw stderr (level 3 only, redacted)
    pub raw_stderr: Option<String>,
}

impl ProbeTrace {
    pub fn new(id: &str, command: &str, exit_code: i32, duration_ms: u64) -> Self {
        Self {
            id: id.to_string(),
            command: command.to_string(),
            exit_code,
            duration_ms,
            parsed: HashMap::new(),
            raw_stdout: None,
            raw_stderr: None,
        }
    }

    /// Add parsed key-value.
    pub fn add_parsed(&mut self, key: &str, value: &str) {
        self.parsed.insert(key.to_string(), value.to_string());
    }

    /// Set raw output (will be redacted).
    pub fn with_raw(mut self, stdout: &str, stderr: &str, redactor: &Redactor) -> Self {
        self.raw_stdout = Some(redactor.redact(stdout));
        if !stderr.is_empty() {
            self.raw_stderr = Some(redactor.redact(stderr));
        }
        self
    }
}

/// LLM call trace info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTrace {
    /// Role (translator/specialist/verifier)
    pub role: String,
    /// Model name
    pub model: String,
    /// Duration in ms
    pub duration_ms: u64,
    /// Input token estimate
    pub input_tokens_est: u32,
    /// Output token estimate
    pub output_tokens_est: u32,
    /// Temperature used
    pub temperature: f32,
    /// Max tokens setting
    pub max_tokens: u32,
    /// Parse success
    pub parse_success: bool,
    /// Parse error details (if any)
    pub parse_error: Option<ParseErrorInfo>,
    /// Prompt digest (level 2)
    pub prompt_digest: Option<PromptDigest>,
    /// Full prompt (level 3, redacted)
    pub full_prompt: Option<String>,
    /// Full response (level 3, redacted, even if invalid JSON)
    pub full_response: Option<String>,
}

impl LlmTrace {
    pub fn new(role: &str, model: &str) -> Self {
        Self {
            role: role.to_string(),
            model: model.to_string(),
            duration_ms: 0,
            input_tokens_est: 0,
            output_tokens_est: 0,
            temperature: 0.0,
            max_tokens: 0,
            parse_success: false,
            parse_error: None,
            prompt_digest: None,
            full_prompt: None,
            full_response: None,
        }
    }

    /// Set timing.
    pub fn with_timing(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Set token estimates.
    pub fn with_tokens(mut self, input: u32, output: u32) -> Self {
        self.input_tokens_est = input;
        self.output_tokens_est = output;
        self
    }

    /// Set model params.
    pub fn with_params(mut self, temperature: f32, max_tokens: u32) -> Self {
        self.temperature = temperature;
        self.max_tokens = max_tokens;
        self
    }

    /// Set parse result.
    pub fn with_parse(mut self, success: bool, error: Option<ParseErrorInfo>) -> Self {
        self.parse_success = success;
        self.parse_error = error;
        self
    }

    /// Set prompt digest (level 2).
    pub fn with_digest(mut self, system: &str, user: &str) -> Self {
        self.prompt_digest = Some(PromptDigest::new(system, user));
        self
    }

    /// Set full prompt/response (level 3, redacted).
    pub fn with_full(mut self, prompt: &str, response: &str, redactor: &Redactor) -> Self {
        self.full_prompt = Some(redactor.redact(prompt));
        self.full_response = Some(redactor.redact(response));
        self
    }
}

/// Parse error details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseErrorInfo {
    /// Error message
    pub message: String,
    /// Byte offset where error occurred
    pub byte_offset: Option<usize>,
    /// Field name that failed
    pub field_name: Option<String>,
    /// Raw output snippet around error
    pub context: Option<String>,
}

impl ParseErrorInfo {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            byte_offset: None,
            field_name: None,
            context: None,
        }
    }

    pub fn with_location(mut self, offset: usize, field: &str) -> Self {
        self.byte_offset = Some(offset);
        self.field_name = Some(field.to_string());
        self
    }

    pub fn with_context(mut self, raw: &str, offset: usize) -> Self {
        // Extract ~100 chars around the error
        let start = offset.saturating_sub(50);
        let end = (offset + 50).min(raw.len());
        if let Some(slice) = raw.get(start..end) {
            self.context = Some(slice.to_string());
        }
        self
    }
}

/// Compact prompt digest (level 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDigest {
    /// Hash of system prompt
    pub system_hash: String,
    /// Hash of user prompt
    pub user_hash: String,
    /// First 200 chars of system prompt
    pub system_preview: String,
    /// First 200 chars of user prompt
    pub user_preview: String,
    /// Total prompt length
    pub total_chars: usize,
}

impl PromptDigest {
    pub fn new(system: &str, user: &str) -> Self {
        Self {
            system_hash: simple_hash(system),
            user_hash: simple_hash(user),
            system_preview: system.chars().take(200).collect(),
            user_preview: user.chars().take(200).collect(),
            total_chars: system.len() + user.len(),
        }
    }
}

/// Simple hash for prompt digests.
fn simple_hash(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// Timing breakdown.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimingTrace {
    pub translator_ms: u64,
    pub probes_ms: u64,
    pub specialist_ms: u64,
    pub gate_ms: u64,
    pub total_ms: u64,
}

/// Timeout information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutInfo {
    /// Which stage timed out
    pub stage: String,
    /// Configured timeout (ms)
    pub timeout_ms: u64,
    /// Elapsed time when timeout occurred
    pub elapsed_ms: u64,
    /// Partial output captured (level 3)
    pub partial_output: Option<String>,
}

/// Failure detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetail {
    /// Which check failed
    pub check: String,
    /// Why it failed
    pub reason: String,
    /// Additional context
    pub context: Option<String>,
}

/// Reliability gate result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Pass or fail
    pub passed: bool,
    /// Individual check results
    pub checks: Vec<GateCheck>,
}

/// Individual gate check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheck {
    pub name: String,
    pub passed: bool,
    pub details: Option<String>,
}

/// The canonical trace block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceBlock {
    /// Request ID
    pub request_id: String,
    /// Original query
    pub query: String,
    /// Detected intent
    pub intent: String,
    /// Detected domain
    pub domain: String,
    /// Route type
    pub route: RouteType,
    /// Probes run
    pub probes: Vec<String>,
    /// Outcome
    pub outcome: TraceOutcome,
    /// Reliability gate result
    pub reliability_gate: GateResult,
    /// Failures (if any)
    pub failures: Vec<FailureDetail>,
    /// Timing breakdown
    pub timings: TimingTrace,
    /// Timeout info (if occurred)
    pub timeout: Option<TimeoutInfo>,
    /// Probe traces (level >= 2)
    pub probe_traces: Vec<ProbeTrace>,
    /// LLM traces (level >= 2)
    pub llm_traces: Vec<LlmTrace>,
    /// Timestamp
    pub timestamp: u64,
}

impl TraceBlock {
    /// Create new trace block.
    pub fn new(request_id: &str, query: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            query: query.to_string(),
            intent: String::new(),
            domain: String::new(),
            route: RouteType::LlmSpecialist,
            probes: Vec::new(),
            outcome: TraceOutcome::Success,
            reliability_gate: GateResult {
                passed: false,
                checks: Vec::new(),
            },
            failures: Vec::new(),
            timings: TimingTrace::default(),
            timeout: None,
            probe_traces: Vec::new(),
            llm_traces: Vec::new(),
            timestamp: current_millis(),
        }
    }

    /// Set intent/domain.
    pub fn with_classification(mut self, intent: &str, domain: &str) -> Self {
        self.intent = intent.to_string();
        self.domain = domain.to_string();
        self
    }

    /// Set route type.
    pub fn with_route(mut self, route: RouteType) -> Self {
        self.route = route;
        self
    }

    /// Add probe names.
    pub fn add_probe(&mut self, probe: &str) {
        self.probes.push(probe.to_string());
    }

    /// Set outcome.
    pub fn with_outcome(mut self, outcome: TraceOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Add failure detail.
    pub fn add_failure(&mut self, check: &str, reason: &str) {
        self.failures.push(FailureDetail {
            check: check.to_string(),
            reason: reason.to_string(),
            context: None,
        });
    }

    /// Set timeout info.
    pub fn with_timeout(mut self, stage: &str, timeout_ms: u64, elapsed_ms: u64) -> Self {
        self.timeout = Some(TimeoutInfo {
            stage: stage.to_string(),
            timeout_ms,
            elapsed_ms,
            partial_output: None,
        });
        self.outcome = TraceOutcome::FailedTimeout;
        self
    }

    /// Add probe trace.
    pub fn add_probe_trace(&mut self, trace: ProbeTrace) {
        self.probe_traces.push(trace);
    }

    /// Add LLM trace.
    pub fn add_llm_trace(&mut self, trace: LlmTrace) {
        self.llm_traces.push(trace);
    }

    /// Format for display at given level.
    pub fn format(&self, level: DebugLevel) -> Option<String> {
        if level == DebugLevel::Off {
            return None;
        }

        let mut out = String::new();
        out.push_str("\n[trace]\n");

        // Level 1 (Summary): Basic info
        out.push_str(&format!("  request_id:       {}\n", self.request_id));
        out.push_str(&format!(
            "  intent/domain:    {}/{}\n",
            self.intent, self.domain
        ));
        out.push_str(&format!("  route:            {}\n", self.route));
        out.push_str(&format!(
            "  probes:           [{}]\n",
            self.probes.join(", ")
        ));
        out.push_str(&format!("  outcome:          {}\n", self.outcome));
        out.push_str(&format!(
            "  reliability_gate: {}\n",
            if self.reliability_gate.passed {
                "PASS"
            } else {
                "FAIL"
            }
        ));

        if !self.failures.is_empty() {
            out.push_str("  failures:\n");
            for f in &self.failures {
                out.push_str(&format!("    - {}: {}\n", f.check, f.reason));
            }
        }

        // Timings
        out.push_str("  timings:\n");
        out.push_str(&format!(
            "    translator_ms:  {}\n",
            self.timings.translator_ms
        ));
        out.push_str(&format!("    probes_ms:      {}\n", self.timings.probes_ms));
        out.push_str(&format!(
            "    specialist_ms:  {}\n",
            self.timings.specialist_ms
        ));
        out.push_str(&format!("    gate_ms:        {}\n", self.timings.gate_ms));
        out.push_str(&format!("    total_ms:       {}\n", self.timings.total_ms));

        // Timeout info
        if let Some(t) = &self.timeout {
            out.push_str("  timeout:\n");
            out.push_str(&format!("    stage:          {}\n", t.stage));
            out.push_str(&format!("    configured_ms:  {}\n", t.timeout_ms));
            out.push_str(&format!("    elapsed_ms:     {}\n", t.elapsed_ms));
        }

        // Level 2 (Trace): Probe and LLM details
        if level >= DebugLevel::Trace {
            if !self.probe_traces.is_empty() {
                out.push_str("\n  [probes]\n");
                for p in &self.probe_traces {
                    out.push_str(&format!(
                        "    {} (exit={}, {}ms)\n",
                        p.id, p.exit_code, p.duration_ms
                    ));
                    out.push_str(&format!("      cmd: {}\n", p.command));
                    if !p.parsed.is_empty() {
                        out.push_str("      parsed:\n");
                        for (k, v) in &p.parsed {
                            out.push_str(&format!("        {}: {}\n", k, v));
                        }
                    }
                }
            }

            if !self.llm_traces.is_empty() {
                out.push_str("\n  [llm_calls]\n");
                for l in &self.llm_traces {
                    out.push_str(&format!(
                        "    {} ({}, {}ms)\n",
                        l.role, l.model, l.duration_ms
                    ));
                    out.push_str(&format!(
                        "      tokens: ~{} in, ~{} out\n",
                        l.input_tokens_est, l.output_tokens_est
                    ));
                    out.push_str(&format!(
                        "      params: temp={}, max_tokens={}\n",
                        l.temperature, l.max_tokens
                    ));
                    out.push_str(&format!(
                        "      parse: {}\n",
                        if l.parse_success { "ok" } else { "FAILED" }
                    ));

                    if let Some(err) = &l.parse_error {
                        out.push_str(&format!("      parse_error: {}\n", err.message));
                        if let Some(off) = err.byte_offset {
                            out.push_str(&format!("        at byte: {}\n", off));
                        }
                        if let Some(field) = &err.field_name {
                            out.push_str(&format!("        field: {}\n", field));
                        }
                    }

                    if let Some(digest) = &l.prompt_digest {
                        out.push_str(&format!(
                            "      prompt: {} chars (sys:{}, user:{})\n",
                            digest.total_chars, digest.system_hash, digest.user_hash
                        ));
                    }
                }
            }

            // Gate checks
            if !self.reliability_gate.checks.is_empty() {
                out.push_str("\n  [gate_checks]\n");
                for c in &self.reliability_gate.checks {
                    let status = if c.passed { "PASS" } else { "FAIL" };
                    out.push_str(&format!("    {} {}", status, c.name));
                    if let Some(d) = &c.details {
                        out.push_str(&format!(" ({})", d));
                    }
                    out.push('\n');
                }
            }
        }

        // Level 3 (Full): Raw outputs
        if level >= DebugLevel::Full {
            if !self.probe_traces.is_empty() {
                out.push_str("\n  [probe_raw]\n");
                for p in &self.probe_traces {
                    if let Some(stdout) = &p.raw_stdout {
                        out.push_str(&format!("    --- {} stdout ---\n", p.id));
                        for line in stdout.lines().take(50) {
                            out.push_str(&format!("    {}\n", line));
                        }
                    }
                    if let Some(stderr) = &p.raw_stderr {
                        out.push_str(&format!("    --- {} stderr ---\n", p.id));
                        for line in stderr.lines().take(20) {
                            out.push_str(&format!("    {}\n", line));
                        }
                    }
                }
            }

            if !self.llm_traces.is_empty() {
                out.push_str("\n  [llm_raw]\n");
                for l in &self.llm_traces {
                    if let Some(prompt) = &l.full_prompt {
                        out.push_str(&format!("    --- {} prompt ---\n", l.role));
                        for line in prompt.lines().take(100) {
                            out.push_str(&format!("    {}\n", line));
                        }
                    }
                    if let Some(response) = &l.full_response {
                        out.push_str(&format!("    --- {} response ---\n", l.role));
                        for line in response.lines().take(100) {
                            out.push_str(&format!("    {}\n", line));
                        }
                    }
                }
            }

            // Timeout partial output
            if let Some(t) = &self.timeout {
                if let Some(partial) = &t.partial_output {
                    out.push_str("\n  [timeout_partial]\n");
                    for line in partial.lines().take(50) {
                        out.push_str(&format!("    {}\n", line));
                    }
                }
            }
        }

        Some(out)
    }

    /// Serialize to JSON for storage.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_block_summary() {
        let trace = TraceBlock::new("REQ-001", "how much ram")
            .with_classification("inspect", "system")
            .with_route(RouteType::Deterministic)
            .with_outcome(TraceOutcome::Success);

        let output = trace.format(DebugLevel::Summary);
        assert!(output.is_some());
        let s = output.unwrap();
        assert!(s.contains("REQ-001"));
        assert!(s.contains("inspect/system"));
        assert!(s.contains("deterministic"));
        assert!(s.contains("SUCCESS"));
    }

    #[test]
    fn test_trace_block_with_probes() {
        let mut trace = TraceBlock::new("REQ-002", "disk usage");

        let mut probe = ProbeTrace::new("df", "df -h", 0, 50);
        probe.add_parsed("root_percent", "75%");
        trace.add_probe_trace(probe);

        let output = trace.format(DebugLevel::Trace);
        assert!(output.is_some());
        let s = output.unwrap();
        assert!(s.contains("df"));
        assert!(s.contains("root_percent"));
        assert!(s.contains("75%"));
    }

    #[test]
    fn test_trace_block_with_timeout() {
        let trace =
            TraceBlock::new("REQ-003", "complex query").with_timeout("specialist", 10000, 15000);

        let output = trace.format(DebugLevel::Summary);
        assert!(output.is_some());
        let s = output.unwrap();
        assert!(s.contains("FAILED_TIMEOUT"));
        assert!(s.contains("specialist"));
        assert!(s.contains("10000"));
        assert!(s.contains("15000"));
    }

    #[test]
    fn test_parse_error_info() {
        let err = ParseErrorInfo::new("Expected '}' at end of object")
            .with_location(1234, "evidence")
            .with_context("{\"answer\": \"test\", \"evidence\":", 1234);

        assert_eq!(err.byte_offset, Some(1234));
        assert_eq!(err.field_name, Some("evidence".to_string()));
    }

    #[test]
    fn test_prompt_digest() {
        let digest = PromptDigest::new("You are a helpful assistant.", "What is my disk usage?");

        assert!(!digest.system_hash.is_empty());
        assert!(!digest.user_hash.is_empty());
        assert!(digest.total_chars > 0);
    }
}
