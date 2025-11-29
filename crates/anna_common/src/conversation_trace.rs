//! Conversation Trace Module for v1.0.0 - "Anna, the Movie"
//!
//! Provides a "fly on the wall" view of how Anna, Junior, and Senior reason
//! about questions. Designed for debug mode with zero extra LLM calls.
//!
//! Key principles:
//! - All narrative text from static templates, never LLM-generated
//! - O(probes + iterations) complexity, simple string formatting
//! - Captures semantic reasoning, not raw JSON blobs

use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// Answer Origin - Where did the answer come from?
// ============================================================================

/// Origin of the final answer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnswerOrigin {
    /// Brain fast path - no LLM needed
    Brain,
    /// Junior drafted, Senior approved/fixed
    JuniorSenior,
    /// Junior only (Senior skipped due to high confidence)
    JuniorOnly,
    /// Partial answer (incomplete but honest)
    Partial,
    /// Fallback (error recovery)
    Fallback,
    /// Timeout (hard limit reached)
    Timeout,
    /// Refusal (question outside scope)
    Refusal,
}

impl AnswerOrigin {
    /// Short label for display
    pub fn label(&self) -> &'static str {
        match self {
            AnswerOrigin::Brain => "Brain",
            AnswerOrigin::JuniorSenior => "Junior+Senior",
            AnswerOrigin::JuniorOnly => "Junior",
            AnswerOrigin::Partial => "Partial",
            AnswerOrigin::Fallback => "Fallback",
            AnswerOrigin::Timeout => "Timeout",
            AnswerOrigin::Refusal => "Refusal",
        }
    }
}

// ============================================================================
// Probe Trace - What probes ran?
// ============================================================================

/// Trace of a single probe execution (v1.0.0)
/// Named ProbeExecTrace to avoid conflict with trace::ProbeTrace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeExecTrace {
    /// Probe identifier (e.g., "cpu.info", "mem.info")
    pub probe_id: String,
    /// Execution status
    pub status: ProbeStatus,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Brief description of what was found (optional)
    pub summary: Option<String>,
}

/// Status of probe execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbeStatus {
    Ok,
    Error,
    Timeout,
    Skipped,
}

impl ProbeStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ProbeStatus::Ok => "ok",
            ProbeStatus::Error => "error",
            ProbeStatus::Timeout => "timeout",
            ProbeStatus::Skipped => "skipped",
        }
    }
}

// ============================================================================
// Junior Plan Trace - What did Junior decide?
// ============================================================================

/// Trace of Junior's planning phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuniorPlanTrace {
    /// Probes that Junior requested
    pub requested_probes: Vec<String>,
    /// Whether Junior produced a draft answer
    pub had_draft: bool,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Brief reason for the plan
    pub reason: String,
    /// Duration of Junior's LLM call(s) in ms
    pub duration_ms: u64,
}

// ============================================================================
// Senior Review Trace - What did Senior decide?
// ============================================================================

/// Trace of Senior's review phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeniorReviewTrace {
    /// Senior's verdict
    pub verdict: SeniorVerdictType,
    /// Whether Senior modified the answer
    pub modified_answer: bool,
    /// Brief notes on the review
    pub notes: String,
    /// Duration of Senior's LLM call in ms
    pub duration_ms: u64,
}

/// Senior's verdict on the draft (v1.0.0)
/// Named SeniorVerdictType to avoid conflict with trace::SeniorVerdict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeniorVerdictType {
    /// Draft was correct and complete
    Approve,
    /// Draft needed fixes, Senior corrected it
    FixAndAccept,
    /// Draft was unsalvageable, refused
    Refuse,
    /// Senior timed out
    Timeout,
    /// Senior was skipped (Junior confidence high enough)
    Skipped,
}

impl SeniorVerdictType {
    pub fn label(&self) -> &'static str {
        match self {
            SeniorVerdictType::Approve => "approve",
            SeniorVerdictType::FixAndAccept => "fix_and_accept",
            SeniorVerdictType::Refuse => "refuse",
            SeniorVerdictType::Timeout => "timeout",
            SeniorVerdictType::Skipped => "skipped",
        }
    }

    /// Parse from string (from LLM response)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "approve" => SeniorVerdictType::Approve,
            "fix_and_accept" => SeniorVerdictType::FixAndAccept,
            "refuse" => SeniorVerdictType::Refuse,
            "timeout" => SeniorVerdictType::Timeout,
            "skipped" => SeniorVerdictType::Skipped,
            _ => SeniorVerdictType::Refuse, // Default to refuse for unknown
        }
    }
}

// ============================================================================
// Orchestration Trace - The complete "movie"
// ============================================================================

/// Complete trace of how a question was answered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationTrace {
    /// The original question
    pub question: String,
    /// Where the answer came from
    pub origin: AnswerOrigin,
    /// Total end-to-end duration in ms
    pub total_duration_ms: u64,
    /// Whether Brain fast path was attempted
    pub brain_attempted: bool,
    /// Why Brain path was used or skipped
    pub brain_reason: Option<String>,
    /// Junior's planning trace (if used)
    pub junior_plan: Option<JuniorPlanTrace>,
    /// Probes that were executed
    pub probes_run: Vec<ProbeExecTrace>,
    /// Senior's review trace (if used)
    pub senior_review: Option<SeniorReviewTrace>,
    /// Number of orchestration iterations
    pub iterations: u32,
    /// Any error or timeout that occurred
    pub failure_reason: Option<String>,
}

impl Default for OrchestrationTrace {
    fn default() -> Self {
        Self {
            question: String::new(),
            origin: AnswerOrigin::Fallback,
            total_duration_ms: 0,
            brain_attempted: false,
            brain_reason: None,
            junior_plan: None,
            probes_run: Vec::new(),
            senior_review: None,
            iterations: 0,
            failure_reason: None,
        }
    }
}

impl OrchestrationTrace {
    /// Create a new trace for a question
    pub fn new(question: &str) -> Self {
        Self {
            question: question.to_string(),
            ..Default::default()
        }
    }

    /// Record Brain fast path attempt
    pub fn set_brain_attempt(&mut self, used: bool, reason: &str) {
        self.brain_attempted = true;
        self.brain_reason = Some(reason.to_string());
        if used {
            self.origin = AnswerOrigin::Brain;
        }
    }

    /// Record Junior's plan
    pub fn set_junior_plan(
        &mut self,
        requested_probes: Vec<String>,
        had_draft: bool,
        confidence: u8,
        reason: &str,
        duration_ms: u64,
    ) {
        self.junior_plan = Some(JuniorPlanTrace {
            requested_probes,
            had_draft,
            confidence,
            reason: reason.to_string(),
            duration_ms,
        });
    }

    /// Add a probe execution result
    pub fn add_probe(&mut self, probe_id: &str, status: ProbeStatus, duration_ms: u64) {
        self.probes_run.push(ProbeExecTrace {
            probe_id: probe_id.to_string(),
            status,
            duration_ms,
            summary: None,
        });
    }

    /// Add a probe with summary
    pub fn add_probe_with_summary(
        &mut self,
        probe_id: &str,
        status: ProbeStatus,
        duration_ms: u64,
        summary: &str,
    ) {
        self.probes_run.push(ProbeExecTrace {
            probe_id: probe_id.to_string(),
            status,
            duration_ms,
            summary: Some(summary.to_string()),
        });
    }

    /// Record Senior's review
    pub fn set_senior_review(
        &mut self,
        verdict: SeniorVerdictType,
        modified_answer: bool,
        notes: &str,
        duration_ms: u64,
    ) {
        self.senior_review = Some(SeniorReviewTrace {
            verdict,
            modified_answer,
            notes: notes.to_string(),
            duration_ms,
        });
        // Update origin based on Senior involvement
        if verdict != SeniorVerdictType::Skipped {
            self.origin = AnswerOrigin::JuniorSenior;
        } else {
            self.origin = AnswerOrigin::JuniorOnly;
        }
    }

    /// Mark as timeout
    pub fn set_timeout(&mut self, reason: &str) {
        self.origin = AnswerOrigin::Timeout;
        self.failure_reason = Some(reason.to_string());
    }

    /// Mark as partial answer
    pub fn set_partial(&mut self, reason: &str) {
        self.origin = AnswerOrigin::Partial;
        self.failure_reason = Some(reason.to_string());
    }

    /// Mark as refusal
    pub fn set_refusal(&mut self, reason: &str) {
        self.origin = AnswerOrigin::Refusal;
        self.failure_reason = Some(reason.to_string());
    }

    /// Finalize with total duration
    pub fn finalize(&mut self, total_duration_ms: u64) {
        self.total_duration_ms = total_duration_ms;
    }
}

// ============================================================================
// Narrative Formatter - "Fly on the Wall" View
// ============================================================================

impl OrchestrationTrace {
    /// Generate human-readable conversation trace (no LLM calls!)
    /// This is the "fly on the wall" narrative.
    pub fn to_narrative(&self) -> String {
        let mut lines = Vec::new();

        lines.push("[DEBUG] Conversation trace".to_string());
        lines.push(String::new());

        // Brain path
        if self.brain_attempted {
            if self.origin == AnswerOrigin::Brain {
                let reason = self.brain_reason.as_deref().unwrap_or("simple question");
                lines.push(format!(
                    "Anna (Brain): \"This looks like a {}, I can answer without LLM.\"",
                    reason
                ));
            } else {
                let reason = self.brain_reason.as_deref().unwrap_or("complex question");
                lines.push(format!(
                    "Anna: \"Brain cannot handle this ({})—routing to Junior.\"",
                    reason
                ));
            }
        }

        // Junior plan
        if let Some(ref plan) = self.junior_plan {
            if !plan.requested_probes.is_empty() {
                let probes = plan.requested_probes.join(", ");
                lines.push(format!(
                    "Junior: \"I will run these probes: [{}].\"",
                    probes
                ));
            }
            if plan.had_draft {
                lines.push(format!(
                    "Junior: \"With this data I can draft an answer. Confidence: {}/100.\"",
                    plan.confidence
                ));
            } else {
                lines.push("Junior: \"I do not have enough evidence for a draft yet.\"".to_string());
            }
        }

        // Probes executed
        if !self.probes_run.is_empty() {
            lines.push("Probes run:".to_string());
            for probe in &self.probes_run {
                let status_str = probe.status.label();
                lines.push(format!(
                    "  - {} ({}, {}ms)",
                    probe.probe_id, status_str, probe.duration_ms
                ));
            }
        }

        // Senior review
        if let Some(ref review) = self.senior_review {
            match review.verdict {
                SeniorVerdictType::Approve => {
                    lines.push(
                        "Senior: \"Draft is correct and complete. Verdict: approve.\"".to_string(),
                    );
                }
                SeniorVerdictType::FixAndAccept => {
                    if review.modified_answer {
                        lines.push(format!(
                            "Senior: \"{} Verdict: fix_and_accept.\"",
                            review.notes
                        ));
                    } else {
                        lines.push(
                            "Senior: \"Draft needed minor adjustments. Verdict: fix_and_accept.\""
                                .to_string(),
                        );
                    }
                }
                SeniorVerdictType::Refuse => {
                    lines.push(format!(
                        "Senior: \"Cannot verify this answer. {} Verdict: refuse.\"",
                        review.notes
                    ));
                }
                SeniorVerdictType::Timeout => {
                    lines.push(
                        "Senior: \"Timed out before completing review.\"".to_string(),
                    );
                }
                SeniorVerdictType::Skipped => {
                    lines.push(
                        "Anna: \"Junior's confidence was high enough—skipping Senior review.\""
                            .to_string(),
                    );
                }
            }
        }

        // Result summary
        lines.push("Result:".to_string());
        match self.origin {
            AnswerOrigin::Brain => {
                lines.push("  - Answered directly from Brain with high confidence.".to_string());
            }
            AnswerOrigin::JuniorSenior => {
                lines.push(
                    "  - Answer synthesized from probes and Senior's verification.".to_string(),
                );
            }
            AnswerOrigin::JuniorOnly => {
                lines.push(
                    "  - Answer from Junior (Senior skipped due to high confidence).".to_string(),
                );
            }
            AnswerOrigin::Partial => {
                let reason = self.failure_reason.as_deref().unwrap_or("incomplete evidence");
                lines.push(format!("  - Partial answer provided ({}).", reason));
            }
            AnswerOrigin::Fallback => {
                let reason = self.failure_reason.as_deref().unwrap_or("error recovery");
                lines.push(format!("  - Fallback answer ({}).", reason));
            }
            AnswerOrigin::Timeout => {
                let reason = self.failure_reason.as_deref().unwrap_or("time budget exceeded");
                lines.push(format!("  - Timeout: {}.", reason));
            }
            AnswerOrigin::Refusal => {
                let reason = self.failure_reason.as_deref().unwrap_or("outside scope");
                lines.push(format!("  - Refused: {}.", reason));
            }
        }

        lines.push(String::new());
        lines.push(format!(
            "Total duration: {}ms | Origin: {}",
            self.total_duration_ms,
            self.origin.label()
        ));

        lines.join("\n")
    }

    /// Compact probe list for explain-last-answer
    pub fn probe_summary(&self) -> String {
        if self.probes_run.is_empty() {
            return "No probes were executed.".to_string();
        }

        let mut lines = vec!["Probes:".to_string()];
        for probe in &self.probes_run {
            lines.push(format!(
                "  - {:16} {:8} {}ms",
                probe.probe_id,
                probe.status.label(),
                probe.duration_ms
            ));
        }
        lines.join("\n")
    }
}

// ============================================================================
// Final Answer Display - Unified UX
// ============================================================================

/// Reliability level for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReliabilityLevel {
    Green,  // >= 90%
    Yellow, // 70-89%
    Red,    // < 70%
}

impl ReliabilityLevel {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            ReliabilityLevel::Green
        } else if score >= 0.7 {
            ReliabilityLevel::Yellow
        } else {
            ReliabilityLevel::Red
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ReliabilityLevel::Green => "Green",
            ReliabilityLevel::Yellow => "Yellow",
            ReliabilityLevel::Red => "Red",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            ReliabilityLevel::Green => "green",
            ReliabilityLevel::Yellow => "yellow",
            ReliabilityLevel::Red => "red",
        }
    }
}

/// Final answer for unified display
#[derive(Debug, Clone)]
pub struct FinalAnswerDisplay {
    /// The answer text
    pub text: String,
    /// Reliability score (0.0 - 1.0)
    pub reliability: f64,
    /// Where the answer came from
    pub origin: AnswerOrigin,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Whether this is a partial/incomplete answer
    pub is_partial: bool,
    /// Whether this is a refusal
    pub is_refusal: bool,
    /// The orchestration trace (for debug mode)
    pub trace: Option<OrchestrationTrace>,
}

impl FinalAnswerDisplay {
    /// Create from components
    pub fn new(
        text: String,
        reliability: f64,
        origin: AnswerOrigin,
        duration_ms: u64,
    ) -> Self {
        Self {
            text,
            reliability,
            origin,
            duration_ms,
            is_partial: origin == AnswerOrigin::Partial,
            is_refusal: origin == AnswerOrigin::Refusal,
            trace: None,
        }
    }

    /// Attach trace for debug mode
    pub fn with_trace(mut self, trace: OrchestrationTrace) -> Self {
        self.trace = Some(trace);
        self
    }

    /// Get reliability level
    pub fn reliability_level(&self) -> ReliabilityLevel {
        ReliabilityLevel::from_score(self.reliability)
    }

    /// Format reliability as percentage string
    pub fn reliability_pct(&self) -> String {
        format!("{}%", (self.reliability * 100.0).round() as u32)
    }

    /// Generate the unified answer display (no colors, plain text)
    pub fn format_plain(&self) -> String {
        let mut lines = Vec::new();

        lines.push("------------------------------------------------------------".to_string());
        lines.push("Anna".to_string());
        lines.push("------------------------------------------------------------".to_string());
        lines.push(String::new());
        lines.push(self.text.clone());
        lines.push(String::new());
        lines.push("------------------------------------------------------------".to_string());
        lines.push(format!(
            "Reliability: {} ({}) | Origin: {} | Duration: {}ms",
            self.reliability_pct(),
            self.reliability_level().label(),
            self.origin.label(),
            self.duration_ms
        ));
        lines.push("------------------------------------------------------------".to_string());

        lines.join("\n")
    }

    /// Generate answer display with debug trace (if available and debug is on)
    pub fn format_with_debug(&self, debug_enabled: bool) -> String {
        let mut output = self.format_plain();

        if debug_enabled {
            if let Some(ref trace) = self.trace {
                output.push_str("\n\n");
                output.push_str(&trace.to_narrative());
            }
        }

        output
    }
}

// ============================================================================
// Last Answer Storage - For "explain last answer"
// ============================================================================

use std::sync::Mutex;

lazy_static::lazy_static! {
    /// Global storage for the last answer (for explain-last-answer pattern)
    static ref LAST_ANSWER: Mutex<Option<FinalAnswerDisplay>> = Mutex::new(None);
}

/// Store the last answer for later explanation
pub fn store_last_answer(answer: FinalAnswerDisplay) {
    if let Ok(mut guard) = LAST_ANSWER.lock() {
        *guard = Some(answer);
    }
}

/// Retrieve the last answer for explanation
pub fn get_last_answer() -> Option<FinalAnswerDisplay> {
    LAST_ANSWER.lock().ok().and_then(|guard| guard.clone())
}

/// Clear the last answer
pub fn clear_last_answer() {
    if let Ok(mut guard) = LAST_ANSWER.lock() {
        *guard = None;
    }
}

/// Check if there is a last answer to explain
pub fn has_last_answer() -> bool {
    LAST_ANSWER
        .lock()
        .ok()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

// ============================================================================
// Explain Intent Detection
// ============================================================================

/// Detect if user is asking to explain the last answer
pub fn is_explain_request(question: &str) -> bool {
    let q = question.to_lowercase();

    // Patterns for explain-last-answer
    let patterns = [
        "explain how you answered",
        "explain your last answer",
        "explain the last answer",
        "how did you answer",
        "show the reasoning",
        "show your reasoning",
        "what probes did you",
        "which probes did you",
        "explain your answer",
        "how did you get that answer",
        "walk me through",
        "show me how you",
    ];

    patterns.iter().any(|p| q.contains(p))
}

/// Generate explanation of last answer (no LLM calls!)
pub fn explain_last_answer() -> String {
    match get_last_answer() {
        Some(answer) => {
            let mut lines = Vec::new();

            lines.push("------------------------------------------------------------".to_string());
            lines.push("Anna: Explaining my last answer".to_string());
            lines.push("------------------------------------------------------------".to_string());
            lines.push(String::new());

            // Question recap
            if let Some(ref trace) = answer.trace {
                lines.push(format!("Question: \"{}\"", trace.question));
                lines.push(String::new());
            }

            // Answer recap
            lines.push("Answer:".to_string());
            // Truncate long answers for recap
            let answer_preview = if answer.text.len() > 200 {
                format!("{}...", &answer.text[..200])
            } else {
                answer.text.clone()
            };
            lines.push(format!("  {}", answer_preview));
            lines.push(String::new());

            // Reliability info
            lines.push(format!(
                "Reliability: {} ({}) | Origin: {} | Duration: {}ms",
                answer.reliability_pct(),
                answer.reliability_level().label(),
                answer.origin.label(),
                answer.duration_ms
            ));
            lines.push(String::new());

            // Full trace
            if let Some(ref trace) = answer.trace {
                lines.push(trace.to_narrative());
                lines.push(String::new());
                lines.push(trace.probe_summary());
            } else {
                lines.push("No detailed trace available for this answer.".to_string());
            }

            lines.join("\n")
        }
        None => {
            "Anna: I do not have a recent answer to explain.".to_string()
        }
    }
}

// ============================================================================
// Trace Builder - Convert FinalAnswer to OrchestrationTrace
// ============================================================================

use crate::{FinalAnswer, ConfidenceLevel, EvidenceStatus};

impl OrchestrationTrace {
    /// Build an OrchestrationTrace from a FinalAnswer (post-facto reconstruction)
    ///
    /// This allows us to create the trace from the existing FinalAnswer fields
    /// without modifying the engine internals. The trace is approximate but
    /// captures the key reasoning steps.
    pub fn from_final_answer(answer: &FinalAnswer) -> Self {
        let mut trace = OrchestrationTrace::new(&answer.question);

        // Determine origin from model_used and senior_verdict
        let origin = if answer.model_used.as_deref() == Some("Brain") {
            AnswerOrigin::Brain
        } else if answer.is_refusal {
            AnswerOrigin::Refusal
        } else if answer.failure_cause.as_deref() == Some("timeout_or_latency") {
            AnswerOrigin::Timeout
        } else if answer.senior_verdict.as_deref() == Some("skipped") {
            AnswerOrigin::JuniorOnly
        } else if answer.senior_verdict.is_some() {
            AnswerOrigin::JuniorSenior
        } else if answer.junior_ms > 0 {
            AnswerOrigin::JuniorOnly
        } else {
            AnswerOrigin::Fallback
        };
        trace.origin = origin;

        // Brain path
        let is_brain = origin == AnswerOrigin::Brain;
        let brain_reason = if is_brain {
            "matched Brain fast path"
        } else {
            "routed to Junior+Senior"
        };
        trace.set_brain_attempt(is_brain, brain_reason);

        // Junior plan (if Junior was used)
        if answer.junior_ms > 0 || !answer.junior_probes.is_empty() {
            let confidence = match answer.confidence_level {
                ConfidenceLevel::Green => 90,
                ConfidenceLevel::Yellow => 75,
                ConfidenceLevel::Red => 50,
            };
            trace.set_junior_plan(
                answer.junior_probes.clone(),
                answer.junior_had_draft,
                confidence,
                "selected probes based on question",
                answer.junior_ms,
            );
        }

        // Probes (from citations)
        for citation in &answer.citations {
            let status = match citation.status {
                EvidenceStatus::Ok => ProbeStatus::Ok,
                EvidenceStatus::Error => ProbeStatus::Error,
                EvidenceStatus::Timeout => ProbeStatus::Timeout,
                EvidenceStatus::NotFound => ProbeStatus::Skipped,
            };
            // Approximate duration (not tracked individually in FinalAnswer)
            let probe_duration = 50; // Placeholder
            trace.add_probe(&citation.probe_id, status, probe_duration);
        }

        // Senior review
        if let Some(ref verdict_str) = answer.senior_verdict {
            let verdict = SeniorVerdictType::from_str(verdict_str);
            let modified = verdict == SeniorVerdictType::FixAndAccept;
            let notes = match verdict {
                SeniorVerdictType::Approve => "Draft approved without changes",
                SeniorVerdictType::FixAndAccept => "Minor corrections applied",
                SeniorVerdictType::Refuse => "Answer could not be verified",
                SeniorVerdictType::Timeout => "Senior timed out",
                SeniorVerdictType::Skipped => "Skipped due to high Junior confidence",
            };
            trace.set_senior_review(verdict, modified, notes, answer.senior_ms);
        }

        // Failure handling
        if answer.is_refusal {
            if let Some(ref cause) = answer.failure_cause {
                trace.set_refusal(cause);
            } else {
                trace.set_refusal("question outside scope");
            }
        } else if origin == AnswerOrigin::Timeout {
            trace.set_timeout("time budget exceeded");
        }

        // Total duration (approximate from junior + senior)
        let total_ms = answer.junior_ms + answer.senior_ms;
        trace.finalize(total_ms);
        trace.iterations = answer.loop_iterations as u32;

        trace
    }
}

impl FinalAnswerDisplay {
    /// Build a FinalAnswerDisplay from a FinalAnswer
    pub fn from_final_answer(answer: &FinalAnswer, duration_ms: u64) -> Self {
        let origin = if answer.model_used.as_deref() == Some("Brain") {
            AnswerOrigin::Brain
        } else if answer.is_refusal {
            AnswerOrigin::Refusal
        } else if answer.failure_cause.as_deref() == Some("timeout_or_latency") {
            AnswerOrigin::Timeout
        } else if answer.senior_verdict.as_deref() == Some("skipped") {
            AnswerOrigin::JuniorOnly
        } else if answer.senior_verdict.is_some() {
            AnswerOrigin::JuniorSenior
        } else {
            AnswerOrigin::Fallback
        };

        let trace = OrchestrationTrace::from_final_answer(answer);

        FinalAnswerDisplay {
            text: answer.answer.clone(),
            reliability: answer.scores.overall,
            origin,
            duration_ms,
            is_partial: false,
            is_refusal: answer.is_refusal,
            trace: Some(trace),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_trace() {
        let mut trace = OrchestrationTrace::new("How much RAM do I have?");
        trace.set_brain_attempt(true, "simple hardware question");
        trace.add_probe("mem.info", ProbeStatus::Ok, 5);
        trace.finalize(15);

        let narrative = trace.to_narrative();
        assert!(narrative.contains("Brain"));
        assert!(narrative.contains("simple hardware question"));
        assert!(narrative.contains("mem.info"));
    }

    #[test]
    fn test_junior_senior_trace() {
        let mut trace = OrchestrationTrace::new("What services are failing?");
        trace.set_brain_attempt(false, "complex diagnosis needed");
        trace.set_junior_plan(
            vec!["systemd.status".to_string(), "journalctl.errors".to_string()],
            true,
            75,
            "need to check systemd",
            450,
        );
        trace.add_probe("systemd.status", ProbeStatus::Ok, 120);
        trace.add_probe("journalctl.errors", ProbeStatus::Ok, 230);
        trace.set_senior_review(
            SeniorVerdictType::FixAndAccept,
            true,
            "Added missing service names",
            320,
        );
        trace.finalize(1120);

        let narrative = trace.to_narrative();
        assert!(narrative.contains("Junior"));
        assert!(narrative.contains("Senior"));
        assert!(narrative.contains("fix_and_accept"));
        assert!(narrative.contains("systemd.status"));
    }

    #[test]
    fn test_timeout_trace() {
        let mut trace = OrchestrationTrace::new("Complex question");
        trace.set_brain_attempt(false, "complex");
        trace.set_timeout("Junior exceeded 10s budget");
        trace.finalize(10500);

        let narrative = trace.to_narrative();
        assert!(narrative.contains("Timeout"));
        assert!(narrative.contains("Junior exceeded"));
    }

    #[test]
    fn test_reliability_levels() {
        assert_eq!(ReliabilityLevel::from_score(0.95), ReliabilityLevel::Green);
        assert_eq!(ReliabilityLevel::from_score(0.85), ReliabilityLevel::Yellow);
        assert_eq!(ReliabilityLevel::from_score(0.65), ReliabilityLevel::Red);
    }

    #[test]
    fn test_final_answer_display() {
        let answer = FinalAnswerDisplay::new(
            "Your CPU has 16 cores.".to_string(),
            0.95,
            AnswerOrigin::Brain,
            25,
        );

        let display = answer.format_plain();
        assert!(display.contains("Anna"));
        assert!(display.contains("95%"));
        assert!(display.contains("Green"));
        assert!(display.contains("Brain"));
        assert!(display.contains("25ms"));
    }

    #[test]
    fn test_explain_detection() {
        assert!(is_explain_request("explain how you answered that"));
        assert!(is_explain_request("show the reasoning for your answer"));
        assert!(is_explain_request("what probes did you use?"));
        assert!(!is_explain_request("what is my CPU?"));
        assert!(!is_explain_request("how many cores?"));
    }

    #[test]
    fn test_explain_no_answer() {
        clear_last_answer();
        let explanation = explain_last_answer();
        assert!(explanation.contains("do not have a recent answer"));
    }

    #[test]
    fn test_from_final_answer_brain() {
        use crate::{FinalAnswer, AuditScores, ConfidenceLevel};

        let answer = FinalAnswer {
            question: "How much RAM?".to_string(),
            answer: "16 GB".to_string(),
            is_refusal: false,
            citations: vec![],
            scores: AuditScores::new(0.95, 0.95, 0.95),
            confidence_level: ConfidenceLevel::Green,
            problems: vec![],
            loop_iterations: 0,
            model_used: Some("Brain".to_string()),
            clarification_needed: None,
            debug_trace: None,
            junior_ms: 0,
            senior_ms: 0,
            junior_probes: vec![],
            junior_had_draft: false,
            senior_verdict: None,
            failure_cause: None,
        };

        let trace = OrchestrationTrace::from_final_answer(&answer);
        assert_eq!(trace.origin, AnswerOrigin::Brain);
        assert!(trace.brain_attempted);
        assert!(trace.junior_plan.is_none());
    }

    #[test]
    fn test_from_final_answer_junior_senior() {
        use crate::{FinalAnswer, AuditScores, ConfidenceLevel};

        let answer = FinalAnswer {
            question: "What services failed?".to_string(),
            answer: "nginx and redis".to_string(),
            is_refusal: false,
            citations: vec![],
            scores: AuditScores::new(0.85, 0.85, 0.85),
            confidence_level: ConfidenceLevel::Yellow,
            problems: vec![],
            loop_iterations: 2,
            model_used: Some("qwen2.5:7b".to_string()),
            clarification_needed: None,
            debug_trace: None,
            junior_ms: 1500,
            senior_ms: 800,
            junior_probes: vec!["systemd.status".to_string()],
            junior_had_draft: true,
            senior_verdict: Some("approve".to_string()),
            failure_cause: None,
        };

        let trace = OrchestrationTrace::from_final_answer(&answer);
        assert_eq!(trace.origin, AnswerOrigin::JuniorSenior);
        assert!(trace.junior_plan.is_some());
        assert!(trace.senior_review.is_some());

        let junior = trace.junior_plan.unwrap();
        assert_eq!(junior.requested_probes, vec!["systemd.status"]);
        assert!(junior.had_draft);

        let senior = trace.senior_review.unwrap();
        assert_eq!(senior.verdict, SeniorVerdictType::Approve);
    }

    #[test]
    fn test_final_answer_display_from_final_answer() {
        use crate::{FinalAnswer, AuditScores, ConfidenceLevel};

        let answer = FinalAnswer {
            question: "Test question".to_string(),
            answer: "Test answer".to_string(),
            is_refusal: false,
            citations: vec![],
            scores: AuditScores::new(0.92, 0.92, 0.92),
            confidence_level: ConfidenceLevel::Green,
            problems: vec![],
            loop_iterations: 1,
            model_used: Some("qwen2.5:7b".to_string()),
            clarification_needed: None,
            debug_trace: None,
            junior_ms: 500,
            senior_ms: 300,
            junior_probes: vec!["cpu.info".to_string()],
            junior_had_draft: true,
            senior_verdict: Some("skipped".to_string()),
            failure_cause: None,
        };

        let display = FinalAnswerDisplay::from_final_answer(&answer, 850);
        assert_eq!(display.origin, AnswerOrigin::JuniorOnly);
        assert_eq!(display.duration_ms, 850);
        assert!(!display.is_refusal);
        assert!(display.trace.is_some());
    }
}
