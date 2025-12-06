//! v0.0.67: Service Desk narrative renderer.
//!
//! Provides clean "movie-terminal" output for debug OFF mode.
//! Non-negotiables:
//! - No icons, no emojis, no raw probe output
//! - No question marks in Anna's final text
//! - No "would you like"
//! - Citations for factual guidance

use crate::rpc::ServiceDeskResult;
use crate::transcript::{Actor, TranscriptEventKind};
use crate::ui::colors;
use chrono::{DateTime, Duration, Utc};
use std::io::{self, Write};

/// Render policy determines what gets shown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPolicy {
    /// Debug OFF: Clean movie-terminal output
    Narrative,
    /// Debug ON: Full developer trace (existing behavior)
    Debug,
}

impl RenderPolicy {
    pub fn from_debug_mode(debug: bool) -> Self {
        if debug { Self::Debug } else { Self::Narrative }
    }
}

/// UI verbosity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    Low,
    #[default]
    Normal,
    High,
}

impl Verbosity {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Self::Low,
            "high" => Self::High,
            _ => Self::Normal,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone)]
pub struct UiConfig {
    pub verbosity: Verbosity,
    pub streaming: bool,
    pub narrative: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::Normal,
            streaming: true,
            narrative: true,
        }
    }
}

/// Risk level for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,     // Read-only operations
    Medium,  // Config edits
    High,    // Package installs, system changes
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Case ID generator for consistent formatting
pub fn generate_case_id(seq: u32) -> String {
    let now = Utc::now();
    format!("CN-{}-{:04}", now.format("%Y%m%d"), seq)
}

/// Format time delta in human terms
pub fn format_time_delta(duration: Duration) -> String {
    let secs = duration.num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        let mins = secs / 60;
        format!("{} minute{}", mins, if mins == 1 { "" } else { "s" })
    } else if secs < 86400 {
        let hours = secs / 3600;
        format!("{} hour{}", hours, if hours == 1 { "" } else { "s" })
    } else {
        let days = secs / 86400;
        format!("{} day{}", days, if days == 1 { "" } else { "s" })
    }
}

/// Header block for all Anna outputs
pub fn render_header(hostname: &str, username: &str, version: &str, debug_mode: bool) {
    let mode = if debug_mode { " [debug]" } else { "" };
    println!();
    println!("{}anna v{}{}{}", colors::HEADER, version, mode, colors::RESET);
    println!("{}{}@{}{}", colors::DIM, username, hostname, colors::RESET);
    println!();
}

/// Narrative greeting for REPL entry
pub fn render_greeting(
    username: &str,
    last_interaction: Option<DateTime<Utc>>,
    boot_time_delta: Option<&str>,
    critical_issues: usize,
) {
    print!("Hello {}", username);

    if let Some(last) = last_interaction {
        let now = Utc::now();
        let delta = now.signed_duration_since(last);
        let delta_str = format_time_delta(delta);
        println!(". It's been {} since you checked in.", delta_str);
    } else {
        println!(". First time here.");
    }

    // Show deltas
    if let Some(boot) = boot_time_delta {
        println!("{}System up: {}{}", colors::DIM, boot, colors::RESET);
    }

    if critical_issues > 0 {
        println!(
            "{}Warning: {} critical issue{} detected.{}",
            colors::WARN,
            critical_issues,
            if critical_issues == 1 { "" } else { "s" },
            colors::RESET
        );
    }

    println!();
}

/// Case flow block header
pub fn render_case_start(case_id: &str, domain: &str) {
    println!("{}Case {} created{}", colors::DIM, case_id, colors::RESET);
    println!("Dispatching to {} team.", domain);
}

/// Show evidence collection in progress
pub fn render_collecting_evidence() {
    print!("Collecting system evidence");
    io::stdout().flush().ok();
}

/// Show evidence collected
pub fn render_evidence_collected(probe_count: usize) {
    println!(
        " {} {} source{} checked.",
        colors::OK,
        probe_count,
        if probe_count == 1 { "" } else { "s" }
    );
}

/// Render internal notes excerpt (short, professional)
pub fn render_internal_notes(notes: &str) {
    if !notes.is_empty() {
        println!("{}Internal notes: {}{}", colors::DIM, notes, colors::RESET);
    }
}

/// Render resolution (final answer)
pub fn render_resolution(answer: &str) {
    println!();
    println!("{}Resolution:{}", colors::HEADER, colors::RESET);
    for line in answer.lines() {
        println!("  {}", line);
    }
}

/// Render clarification options as numbered list ending with period
pub fn render_clarification(prompt: &str, options: &[(String, String)]) {
    println!();
    // Remove any trailing question mark from prompt
    let clean_prompt = prompt.trim_end_matches('?').trim();
    println!("{}{}:{}", colors::HEADER, clean_prompt, colors::RESET);

    for (i, (_key, label)) in options.iter().enumerate() {
        println!("  {}) {}", i + 1, label);
    }
    println!("Reply with the number.");
}

/// Render risk and reliability line
pub fn render_reliability_line(
    reliability: u8,
    risk: RiskLevel,
    evidence_kinds: &[String],
) {
    let evidence_str = if evidence_kinds.is_empty() {
        "none".to_string()
    } else {
        evidence_kinds.join(", ")
    };

    println!(
        "{}reliability: {}%   risk: {}   evidence: {}{}",
        colors::DIM,
        reliability,
        risk.as_str(),
        evidence_str,
        colors::RESET
    );
}

/// Render citation
pub fn render_citation(source: &str, _topic: &str) {
    println!("{}[source: {}]{}", colors::DIM, source, colors::RESET);
}

/// Render uncited warning
pub fn render_uncited() {
    println!("{}[uncited - verification ticket created]{}", colors::WARN, colors::RESET);
}

/// Spinner animation state
pub struct Spinner {
    frames: &'static [&'static str],
    current: usize,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: &["-", "\\", "|", "/"],
            current: 0,
        }
    }

    pub fn tick(&mut self) {
        print!("\r{} ", self.frames[self.current]);
        io::stdout().flush().ok();
        self.current = (self.current + 1) % self.frames.len();
    }

    pub fn clear(&self) {
        print!("\r  \r");
        io::stdout().flush().ok();
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress renderer for streaming updates
pub struct ProgressRenderer {
    policy: RenderPolicy,
    spinner: Spinner,
    stage: Option<String>,
}

impl ProgressRenderer {
    pub fn new(policy: RenderPolicy) -> Self {
        Self {
            policy,
            spinner: Spinner::new(),
            stage: None,
        }
    }

    pub fn show_stage(&mut self, stage: &str) {
        if self.policy == RenderPolicy::Narrative {
            self.spinner.clear();
            println!("{}...{}{}", colors::DIM, stage, colors::RESET);
        }
        self.stage = Some(stage.to_string());
    }

    pub fn tick(&mut self) {
        if self.policy == RenderPolicy::Narrative {
            self.spinner.tick();
        }
    }

    pub fn complete(&mut self) {
        if self.policy == RenderPolicy::Narrative {
            self.spinner.clear();
        }
    }
}

/// Full narrative render for a result (debug OFF)
pub fn render_narrative(result: &ServiceDeskResult, case_seq: u32) {
    let case_id = generate_case_id(case_seq);

    // Show user query
    for event in &result.transcript.events {
        if let TranscriptEventKind::Message { text } = &event.kind {
            if event.from == Actor::You {
                println!("{}you:{} {}", colors::CYAN, colors::RESET, text);
                break;
            }
        }
    }
    println!();

    // Case header
    render_case_start(&case_id, &result.domain.to_string());

    // Evidence count
    let probe_count = result.evidence.probes_executed.len();
    if probe_count > 0 {
        print!("Collecting system evidence");
        println!(
            " {} {} source{} checked.",
            colors::DIM,
            probe_count,
            if probe_count == 1 { "" } else { "s" }
        );
    }

    // Resolution
    let answer = get_answer_text(result);
    render_resolution(&answer);

    // Clarification if needed (ends with period, not question)
    if result.needs_clarification {
        if let Some(ref req) = result.clarification_request {
            let options: Vec<(String, String)> = req.options.iter()
                .map(|o| (o.key.to_string(), o.label.clone()))
                .collect();
            if !options.is_empty() {
                render_clarification(&req.question, &options);
            }
        }
    }

    println!();

    // Risk level based on action type
    let risk = determine_risk_level(&result.answer);

    // Evidence kinds
    let evidence_kinds: Vec<String> = if let Some(trace) = &result.execution_trace {
        trace.evidence_kinds.iter().map(|k| format!("{:?}", k)).collect()
    } else {
        vec![]
    };

    render_reliability_line(result.reliability_score, risk, &evidence_kinds);
}

/// Get final answer text from result
fn get_answer_text(result: &ServiceDeskResult) -> String {
    // Check transcript for FinalAnswer first
    for event in &result.transcript.events {
        if let TranscriptEventKind::FinalAnswer { text } = &event.kind {
            return text.clone();
        }
    }

    // Fall back to clarification or answer
    if result.needs_clarification {
        result.clarification_question.clone().unwrap_or_else(|| result.answer.clone())
    } else {
        result.answer.clone()
    }
}

/// Determine risk level from answer content
fn determine_risk_level(answer: &str) -> RiskLevel {
    let lower = answer.to_lowercase();

    // High risk indicators
    if lower.contains("install") || lower.contains("remove") ||
       lower.contains("pacman") || lower.contains("systemctl enable") ||
       lower.contains("systemctl disable") {
        return RiskLevel::High;
    }

    // Medium risk indicators
    if lower.contains("edit") || lower.contains("modify") ||
       lower.contains("config") || lower.contains("~/.") ||
       lower.contains("/etc/") {
        return RiskLevel::Medium;
    }

    // Default to low (read-only)
    RiskLevel::Low
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_case_id() {
        let id = generate_case_id(42);
        assert!(id.starts_with("CN-"));
        assert!(id.contains("-0042"));
    }

    #[test]
    fn test_format_time_delta() {
        assert_eq!(format_time_delta(Duration::seconds(30)), "just now");
        assert_eq!(format_time_delta(Duration::seconds(120)), "2 minutes");
        assert_eq!(format_time_delta(Duration::seconds(3600)), "1 hour");
        assert_eq!(format_time_delta(Duration::seconds(86400)), "1 day");
    }

    #[test]
    fn test_risk_level_detection() {
        assert_eq!(determine_risk_level("pacman -S vim"), RiskLevel::High);
        assert_eq!(determine_risk_level("edit ~/.vimrc"), RiskLevel::Medium);
        assert_eq!(determine_risk_level("memory usage is 4GB"), RiskLevel::Low);
    }

    #[test]
    fn test_verbosity_from_str() {
        assert_eq!(Verbosity::from_str("low"), Verbosity::Low);
        assert_eq!(Verbosity::from_str("HIGH"), Verbosity::High);
        assert_eq!(Verbosity::from_str("normal"), Verbosity::Normal);
        assert_eq!(Verbosity::from_str("invalid"), Verbosity::Normal);
    }
}
