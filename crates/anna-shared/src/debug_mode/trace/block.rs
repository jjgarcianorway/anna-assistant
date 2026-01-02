//! Canonical trace block.
//!
//! The main TraceBlock structure that aggregates all trace information
//! for a single request, with formatting support for different debug levels.

use super::llm::LlmTrace;
use super::probe::ProbeTrace;
use super::types::{FailureDetail, GateResult, RouteType, TimeoutInfo, TimingTrace, TraceOutcome};
use crate::debug_mode::config::DebugLevel;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

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
}
