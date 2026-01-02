//! Complete debug block for appending to responses.
//!
//! This is the main debug output structure that formats information
//! at different debug levels (SUMMARY, TRACE, FULL).

use super::reason_codes::{ReasonCode, ReasonCodes};
use super::types::{
    EvidenceDebug, LlmCallDebug, ModelsUsedDebug, ProbeDebugInfo, TimeoutDebug, TimingDebug,
    TranslatorDecision,
};
use crate::reliability_metrics::CanonicalOutcome;
use serde::{Deserialize, Serialize};

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
}
